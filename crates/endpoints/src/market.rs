use crate::types::Market;
use anyhow::{Error, Result};

#[derive(Debug, Clone)]
pub struct MarketEndpoint {
    api_base: String,
    client: reqwest::Client,
}

impl MarketEndpoint {
    pub fn new(api_base: &str) -> Result<Self> {
        let client = reqwest::Client::builder().build()?;
        Ok(Self { api_base: api_base.to_string(), client })
    }
}

impl MarketEndpoint {
    pub async fn get_market_by_slug(&self, slug: &str) -> Result<Market, Error> {
        let url = format!("{}/markets/slug/{}", self.api_base, slug);
        let request = self.client.get(&url);

        let response = request.send().await?;
        let status = response.status();

        if status.is_success() {
            response.json().await.map_err(|e| e.into())
        } else {
            let message = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());

            Err(Error::msg(format!("API error: {} ({})", message, status.as_u16())))
        }
    }
}
