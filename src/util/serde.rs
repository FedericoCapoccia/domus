use serde::{Deserialize, Deserializer};

pub fn deserialize_normalized_email<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.trim().to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::deserialize_normalized_email;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct TestPayload {
        #[serde(deserialize_with = "deserialize_normalized_email")]
        email: String,
    }

    #[test]
    fn deserialize_normalized_email_trims_and_lowercases() {
        let payload: TestPayload =
            serde_json::from_str(r#"{ "email": "  USER@Example.COM  " }"#).unwrap();
        assert_eq!(payload.email, "user@example.com");
    }

    #[test]
    fn deserialize_normalized_email_preserves_normalized_email() {
        let payload: TestPayload =
            serde_json::from_str(r#"{ "email": "user@example.com" }"#).unwrap();

        assert_eq!(payload.email, "user@example.com");
    }

    #[test]
    fn deserialize_normalized_email_allows_empty_after_trim() {
        let payload: TestPayload = serde_json::from_str(r#"{ "email": "   " }"#).unwrap();

        assert_eq!(payload.email, "");
    }
}
