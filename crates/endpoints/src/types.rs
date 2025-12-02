use crate::utils::{
    deserialize_decimal_vec_from_json_string, deserialize_string_vec_from_json_string,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Market {
    pub id: String,
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    pub slug: Option<String>,
    #[serde(rename = "outcomePrices")]
    #[serde(deserialize_with = "deserialize_decimal_vec_from_json_string")]
    pub outcome_prices: Vec<Decimal>,
    #[serde(rename = "startDate")]
    pub start_date: Option<DateTime<Utc>>,
    #[serde(rename = "endDate")]
    pub end_date: Option<DateTime<Utc>>,
    #[serde(rename = "clobTokenIds")]
    #[serde(deserialize_with = "deserialize_string_vec_from_json_string")]
    pub clob_token_ids: Vec<String>,
}

impl Default for Market {
    fn default() -> Self {
        Self {
            id: String::new(),
            condition_id: String::new(),
            slug: None,
            outcome_prices: Vec::new(),
            start_date: None,
            end_date: None,
            clob_token_ids: Vec::new(),
        }
    }
}
