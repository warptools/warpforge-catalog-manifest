use std::fmt::Display;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum CatalogModuleCapsule {
    #[serde(rename = "catalogmodule.v1")]
    V1(CatalogModule),
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct CatalogModule {
    pub name: String,
    pub releases: IndexMap<String, String>,
    pub metadata: IndexMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct CatalogRelease {
    #[serde(rename = "releaseName")]
    pub name: String,
    pub items: IndexMap<String, String>,
    pub metadata: IndexMap<String, String>,
}

pub struct ReleaseItem {
    pub module: String,
    pub version: String,
    pub name: String,
}

impl Display for ReleaseItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.module, self.version, self.name)
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
            "releaseName": "v5.1.16",
            "items": {
                "src": "tar:5K7rekQyv4YJphfwfssRsLqHtrL4G9bVmCuarnJyvNaCWzABt6ujLvRRQ48ppRqvNZ",
                "amd64": "tar:12KZBfkSbUHo9JJojPY7HHNP2FMobgTzoefARF2JkBo6KWj2E1mTquL4pfTAKtdfe6"
            },
            "metadata": {
                "replay": "zM5K3aMARrWToyXjaFxxxWmYU7dZUmYp7ir5hDQtzDi2LCGPtw9PNVch9DTts9ApRyPSacJ"
            }
        }"#]];
        let obj: super::CatalogRelease = serde_json::from_str(expect.data).unwrap();
        let reserialized = pretty_json(obj).expect("serialization shouldn't fail");
        expect.assert_eq(&reserialized);
    }
}
