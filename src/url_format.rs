use reqwest::Url;
use serde::{self, Deserialize, Deserializer, Serializer};

pub fn serialize<S>(url: &Url, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    let url_str = url.as_str();
    serializer.serialize_str(url_str)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Url, D::Error>
    where
        D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Url::parse(s.as_str()).map_err(serde::de::Error::custom)
}