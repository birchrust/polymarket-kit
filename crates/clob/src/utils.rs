use anyhow::{Error, Result};
use reqwest::Response;
use std::time::{SystemTime, UNIX_EPOCH};

#[inline]
pub fn get_current_unix_time_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("time went backwards").as_secs()
}

pub async fn into_result<T: serde::de::DeserializeOwned>(resp: Response) -> Result<T> {
    if resp.status().is_success() {
        Ok(resp.json().await?)
    } else {
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap_or_default();
        Err(Error::msg(format!("status{:?}, {:?}", status, text)))
    }
}
