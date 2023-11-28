use core::fmt;
use std::{collections::BTreeSet, fmt::Display};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum CatalogMirrorsCapsule {
    #[serde(rename = "catalogmirrors.v1")]
    V1(CatalogMirrors),
}

impl Display for CatalogMirrorsCapsule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out = serde_json::to_string_pretty(self);
        match out {
            Ok(o) => write!(f, "{}", o),
            Err(_) => Err(fmt::Error::default()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct CatalogMirrors {
    #[serde(rename = "byWare", default)]
    pub by_ware: IndexMap<String, BTreeSet<String>>,
    #[serde(rename = "byModule", default)]
    pub by_module: IndexMap<String, IndexMap<String, BTreeSet<String>>>,
}

impl Display for CatalogMirrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out = serde_json::to_string_pretty(self);
        match out {
            Ok(o) => write!(f, "{}", o),
            Err(_) => Err(fmt::Error::default()),
        }
    }
}

impl Default for CatalogMirrors {
    fn default() -> CatalogMirrors {
        CatalogMirrors {
            by_ware: IndexMap::new(),
            by_module: IndexMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use serde::Serializer;

    fn pretty_json<T: serde::Serialize>(obj: T) -> Result<String, serde_json::Error> {
        let mut buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut serializer = serde_json::Serializer::with_formatter(&mut buf, formatter);
        serializer.serialize_some(&obj)?;
        let s = String::from_utf8(buf).expect("serde_json does not emit non utf8");
        Ok(s)
    }

    #[test]
    fn test_json_roundtrip() {
        let expect = expect![[r#"
        {
            "catalogmirrors.v1": {
                "byWare": {
                    "tar:5K7rekQyv4YJphfwfssRsLqHtrL4G9bVmCuarnJyvNaCWzABt6ujLvRRQ48ppRqvNZ": [
                        "https://ftp.gnu.org/gnu/bash/bash-5.1.16.tar.gz"
                    ]
                },
                "byModule": {
                    "warpsys.org/bash": {
                        "tar": [
                            "ca+https://warpsys-wares.s3.fr-par.scw.cloud"
                        ]
                    }
                }
            }
        }"#]];
        let obj: super::CatalogMirrorsCapsule = serde_json::from_str(expect.data).unwrap();
        let reserialized = pretty_json(obj).expect("serialization shouldn't fail");
        expect.assert_eq(&reserialized);
    }
}
