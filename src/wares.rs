use indexmap::IndexMap;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::error::Error;
use std::path::PathBuf;
use url::Url;

use crate::catalog_mirrors::{CatalogMirrors, CatalogMirrorsCapsule};
use crate::str_error::StrError;
use crate::{mirrors, releases};

pub fn resolve_all(dir: &PathBuf) -> Result<BTreeMap<String, BTreeSet<String>>, Box<dyn Error>> {
    let mirror_data = mirrors::collect(dir)?;
    let release_data = releases::collect(dir)?;
    let result = join(mirror_data, release_data)?;
    Ok(result)
}

fn split_release(release_id: String) -> (String, String, String) {
    let v: Vec<&str> = release_id.splitn(3, ':').collect();
    if v.len() < 3 {
        panic!(r#"expected release id "{release_id}" to have three parts"#);
    };
    return (String::from(v[0]), String::from(v[1]), String::from(v[2]));
}

fn split_ware(ware_id: String) -> (String, String) {
    let v: Vec<&str> = ware_id.splitn(2, ':').collect();
    if v.len() < 2 {
        eprintln!("vector: {:?}", v);
        panic!(r#"expected ware id "{ware_id}" to have two parts"#);
    }
    return (String::from(v[0]), String::from(v[1]));
}

// returns None if mirror is not a content-addressable link.
// otherwise returns the fully-qualified link to the ware for the content-addressable mirror.
fn resolve_ca_link(mirror: String, ware_hash: String) -> Result<Option<String>, Box<dyn Error>> {
    let base = Url::parse(mirror.as_str())?;
    let mut mir_url = base.clone();
    let mut scheme = (&base).scheme();
    if scheme.starts_with("ca+") {
        scheme = &scheme[3..];
    } else if scheme.ends_with("+ca") {
        scheme = &scheme[..scheme.len() - 3];
    } else {
        return Ok(None);
    };
    if ware_hash.len() < 7 {
        return Err(errf!("ware_hash must be at least 7 characters"));
    }
    mir_url
        .path_segments_mut()
        .map_err(|_| "cannot be base")?
        .push(&ware_hash[0..3])
        .push(&ware_hash[3..6])
        .push(&ware_hash);
    // set_scheme will error if the target is a "special" scheme such as https and the prior scheme is not.
    // as such, this string replacement method works just fine.
    let new_url = [scheme, &mir_url[url::Position::AfterScheme..]].join("");
    Ok(Some(new_url))
}

fn join(
    mirrors_capsule: CatalogMirrorsCapsule,
    releases: BTreeMap<String, String>,
) -> Result<BTreeMap<String, BTreeSet<String>>, Box<dyn Error>> {
    let mut result: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut cat_mirrors: CatalogMirrors;
    match mirrors_capsule {
        CatalogMirrorsCapsule::V1(m) => cat_mirrors = m,
    }
    // insert all wares with explicit links
    for (ware_id, loc) in cat_mirrors.by_ware.iter() {
        let (_, ware_hash) = split_ware(ware_id.clone());
        for item in loc.iter() {
            let link_result = resolve_ca_link(item.clone(), ware_hash.clone());
            if let Err(e) = link_result {
                eprintln!("unable to process link for {ware_id}: {item}: {}", e);
                continue;
            };
            let entry = result.entry(ware_id.clone()).or_insert(BTreeSet::new());
            match link_result.unwrap() {
                None => {
                    entry.insert(item.clone());
                }
                Some(l) => {
                    entry.insert(l);
                }
            };
        }
    }

    // create explicit links for by-module
    for (release_id, ware_id) in releases {
        let (module, _, _) = split_release(release_id.clone());
        let (pack_type, ware_hash) = split_ware(ware_id.clone());
        let pack_mirrors = cat_mirrors
            .by_module
            .entry(module.clone())
            .or_insert_with(IndexMap::new)
            .entry(pack_type.clone())
            .or_insert_with(BTreeSet::new);

        let ware_mirrors = result.entry(ware_id).or_insert(BTreeSet::new());
        for mirror in pack_mirrors.iter() {
            if pack_type == "git" {
                // could start handling specific git hosts such as
                // https://github.com/warptools/warpforge/archive/e3ec637a29aee4874de2ce2e70e9c9e85761ce22.zip
                // but that seems somewhat pointless since that is not how the wares are actually retrieved.
                ware_mirrors.insert(mirror.clone());
                continue;
            }
            let link = resolve_ca_link(mirror.clone(), ware_hash.clone())?;
            match link {
                None => {
                    eprintln!("by module mirrors must have content-addressable scheme (e.g. ca+https): {module}:{pack_type} = {mirror}");
                    continue;
                }
                Some(l) => {
                    ware_mirrors.insert(l);
                }
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_split_release() {
        let result = super::split_release("a:b:c:d".into());
        assert_eq!(
            result,
            (String::from("a"), String::from("b"), String::from("c:d"),)
        )
    }

    #[test]
    fn test_split_ware() {
        let result = super::split_ware("a:b:c".into());
        assert_eq!(result, (String::from("a"), String::from("b:c")))
    }

    #[test]
    fn test_resolve_ca_link() {
        let test_cases = vec![
            (
                "ca+http://example.com",
                "abcdefg",
                Some(String::from("http://example.com/abc/def/abcdefg")),
            ),
            (
                "ca+http://example.com/foo",
                "abcdefg",
                Some(String::from("http://example.com/foo/abc/def/abcdefg")),
            ),
            ("http://example.com/foo", "abcdefg", None),
        ];
        for (mirror, ware_hash, expected) in test_cases {
            let result = super::resolve_ca_link(mirror.into(), ware_hash.into()).unwrap();
            assert_eq!(result, expected)
        }
    }
}
