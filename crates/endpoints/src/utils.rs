use rust_decimal::Decimal;
use serde::{de, Deserialize, Deserializer};
use serde_json;
use std::str::FromStr;

pub fn deserialize_decimal_vec_from_json_string<'de, D>(
    deserializer: D,
) -> Result<Vec<Decimal>, D::Error>
where
    D: Deserializer<'de>,
{
    // First, deserialize the outer string
    let json_string = String::deserialize(deserializer)?;

    // Parse the string as a JSON array of strings
    let string_vec: Vec<String> = serde_json::from_str(&json_string)
        .map_err(|e| de::Error::custom(format!("Failed to parse JSON array: {}", e)))?;

    // Convert each string to Decimal
    string_vec
        .into_iter()
        .map(|s| {
            Decimal::from_str(&s).map_err(|e| {
                de::Error::custom(format!("Failed to parse decimal from '{}': {}", s, e))
            })
        })
        .collect()
}

pub fn deserialize_string_vec_from_json_string<'de, D>(
    deserializer: D,
) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    // First, deserialize the outer string
    let json_string = String::deserialize(deserializer)?;

    // Parse the string as a JSON array of strings
    let string_vec: Vec<String> = serde_json::from_str(&json_string)
        .map_err(|e| de::Error::custom(format!("Failed to parse JSON array: {}", e)))?;

    Ok(string_vec)
}
