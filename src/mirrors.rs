use indexmap::IndexMap;
use std::collections::BTreeSet;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path::Path;

use crate::catalog_mirrors::CatalogMirrors;
use crate::catalog_mirrors::CatalogMirrorsCapsule;

fn merge_mirrors(
    a: CatalogMirrorsCapsule,
    b: CatalogMirrorsCapsule,
) -> Result<CatalogMirrorsCapsule, Box<dyn Error>> {
    let mut result: CatalogMirrors;
    match a {
        CatalogMirrorsCapsule::V1(m) => result = m,
        #[allow(unreachable_patterns)]
        _ => unimplemented!("unknown CatalogMirrorCapsule version"),
    }
    let data: CatalogMirrors;
    match b {
        CatalogMirrorsCapsule::V1(m) => data = m,
        #[allow(unreachable_patterns)]
        _ => unimplemented!("unknown CatalogMirrorCapsule version"),
    }
    for (wid, wh_list) in data.by_ware.iter() {
        let entry = result.by_ware.entry(wid.clone()).or_insert(BTreeSet::new());
        entry.extend(wh_list.iter().cloned())
    }
    for (module, inner) in data.by_module.iter() {
        if inner.is_empty() {
            continue;
        }
        let outer = result
            .by_module
            .entry(module.clone())
            .or_insert(IndexMap::new());
        for (packtype, wh_list) in inner.iter() {
            let entry = outer.entry(packtype.clone()).or_insert(BTreeSet::new());
            entry.extend(wh_list.iter().cloned());
        }
    }
    Ok(CatalogMirrorsCapsule::V1(result))
}

fn read_mirrors_json_file(file_path: &Path) -> Result<CatalogMirrorsCapsule, Box<dyn Error>> {
    let mut file = fs::File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let capsule: CatalogMirrorsCapsule = serde_json::from_str(&contents)?;

    Ok(capsule)
}

pub fn collect(dir_path: &Path) -> Result<CatalogMirrorsCapsule, Box<dyn Error>> {
    let mut result: CatalogMirrorsCapsule = CatalogMirrorsCapsule::V1(CatalogMirrors {
        ..Default::default()
    });
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();

        if file_type.is_dir() {
            let sub_result = collect(&path)?;
            result = merge_mirrors(result, sub_result)?;
        } else if file_type.is_file() && path.file_name() == Some(OsStr::new("_mirrors.json")) {
            let data = read_mirrors_json_file(&path);
            match data {
                Ok(n) => {
                    result = merge_mirrors(result, n)?;
                }
                Err(e) => eprintln!("Error: {e}"),
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    type StringIndexMap<V> = indexmap::IndexMap<String, V>;
    use std::collections::BTreeSet;

    macro_rules! string_set {
        ($($s:expr),*) => {
            {
                let mut v : BTreeSet<String> = BTreeSet::new();
                $(
                    v.insert($s.to_string());
                )*
                v
            }
        };
    }
    macro_rules! string_indexmap {
        ( $( $key:expr => $value:expr ),* ) => {
            {
                let mut map = StringIndexMap::new();
                $(
                    map.insert($key.to_string(), $value);
                )*
                map
            }
        };
    }

    #[test]
    fn test_merge() {
        let a = CatalogMirrorsCapsule::V1(CatalogMirrors {
            by_ware: string_indexmap! {"foo" => string_set!["a", "d"]},
            by_module: string_indexmap! {
                "foo"=> string_indexmap!{"bar"=> string_set!["y", "b"]}
            },
        });
        let b = CatalogMirrorsCapsule::V1(CatalogMirrors {
            by_ware: string_indexmap! {"foo" => string_set!["c", "e", "d"], "bar"=> string_set!["b"]},
            by_module: string_indexmap! {
                "foo"=>string_indexmap!{"bar"=>string_set!["x", "a"]},
                "bar"=>string_indexmap!{"grill"=>string_set!["m", "o"]}
            },
        });
        let expect = CatalogMirrorsCapsule::V1(CatalogMirrors {
            by_ware: string_indexmap! {"foo" => string_set!["a","c","d", "e"], "bar"=> string_set!["b"]},
            by_module: string_indexmap! {
            "foo"=> string_indexmap!{"bar"=> string_set!["a", "b", "x", "y"]},
            "bar"=>string_indexmap!{"grill"=>string_set!["m", "o"]}
            },
        });
        let result = merge_mirrors(a, b).unwrap();
        assert_eq!(expect, result, "expected left and got right");
    }
}
