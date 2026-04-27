use serde::{Deserialize, Deserializer};

pub fn deserialize_normalized_email<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.trim().to_lowercase())
}
