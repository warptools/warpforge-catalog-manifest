use std::collections::BTreeMap;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use crate::catalog_releases::{CatalogModule, CatalogModuleCapsule, CatalogRelease};
use crate::errf;
use crate::str_error::StrError; //required by errf macro

const MODULE_FILE_NAME: &str = "_module.json";
const MODULE_RELEASES_DIR_NAME: &str = "_releases";

fn is_module(dir_path: &Path) -> Result<Option<CatalogModule>, Box<dyn Error>> {
    let path = dir_path.join(MODULE_FILE_NAME);
    if let Err(_err) = fs::metadata(&path) {
        Ok(None)
    } else {
        let mut file = fs::File::open(path.as_path())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let capsule: CatalogModuleCapsule = serde_json::from_str(&contents)?;
        match capsule {
            CatalogModuleCapsule::V1(m) => Ok(Some(m)),
        }
    }
}

fn process_module(
    module: CatalogModule,
    module_path: &Path,
) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
    let mut result: BTreeMap<String, String> = BTreeMap::new();
    let releases_path = module_path.join(MODULE_RELEASES_DIR_NAME);
    if let Err(_) = fs::metadata(&releases_path) {
        // missing releases directory
        // TODO: This is likely not the best way to implement this check.
        if module.releases.len() == 0 {
            return Ok(result);
        }
        return Err(errf!(
            "module file contains releases but releases directory does not exist: {}",
            releases_path.to_string_lossy()
        ));
    }
    let mut count = 0;
    for entry in fs::read_dir(releases_path)? {
        count += 1;
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();
        if !file_type.is_file() {
            return Err(errf!(
                r#"releases directory contained a non-regular file "{}""#,
                path.to_string_lossy()
            ));
        }
        let release = read_release_file(path.as_path())?;
        if !module.releases.contains_key(&release.name) {
            eprintln!(
                r#"WARNING: release file "{}" contains release "{}" not found in module releases"#,
                path.display(),
                release.name,
            );
        }
        for (item, ware_id) in release.items.iter() {
            let catalog_ref = format!("{}:{}:{}", module.name, release.name, item);
            if result.contains_key(&catalog_ref) {
                return Err(errf!(
                    r#"malformed catalog: found duplicate catalog ref item: {catalog_ref}"#
                ));
            }
            result.insert(catalog_ref, (*ware_id).clone());
        }
    }
    if count != module.releases.len() {
        eprintln!(
            r#"WARNING: processed {count} release files but expected {} from module "{}""#,
            module.releases.len(),
            module.name,
        )
    }
    Ok(result)
}

fn basename(path: &Path) -> &str {
    let file_name = path.file_name().unwrap_or(&OsStr::new(""));
    file_name.to_str().unwrap_or("")
}

fn read_release_file(path: &Path) -> Result<CatalogRelease, Box<dyn Error>> {
    let mut file = fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let result: CatalogRelease = serde_json::from_str(&contents)?;
    let file_name = basename(path);
    if !file_name.ends_with(".json") {
        return Err(errf!(
            r#"malformed catalog: release file does not end in .json "{}""#,
            path.display()
        ));
    }
    let file_name = file_name.strip_suffix(".json").unwrap();
    if result.name != file_name {
        return Err(errf!(
            r#"malformed catalog: release file "{}" does not have the same name as release "{}""#,
            path.display(),
            result.name
        ));
    }
    Ok(result)
}

pub fn collect(dir_path: &PathBuf) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
    let mut result: BTreeMap<String, String> = BTreeMap::new();
    let module = is_module(dir_path)?;
    if let Some(m) = module {
        let result = process_module(m, dir_path)?;
        return Ok(result);
    }
    // non-modules recurse into sub-directories
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            let sub_result = collect(&entry.path())?;
            for (key, value) in sub_result {
                // copy non-duplicate key/value pairs into result.
                match result.get_mut(&key) {
                    Some(_) => {
                        // I could reasonably do something else here
                        // However, I do not expect to encounter a malformed catalog in the near future.
                        return Err(errf!(
                            "malformed catalog: found duplicate catalog ref item: {key}"
                        ));
                    }
                    None => result.insert(key, value),
                };
            }
        }
    }
    Ok(result)
}
