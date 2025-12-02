use crate::{
    Credentials, OrderType, POLY_ADDR_HEADER, POLY_API_KEY_HEADER, POLY_PASS_HEADER,
    POLY_SIG_HEADER, POLY_TS_HEADER, SignedOrderRequest, get_current_unix_time_secs, into_result,
};
use alloy_primitives::hex::encode_prefixed;
use alloy_signer_local::PrivateKeySigner;
use anyhow::Result;
use base64::{Engine, engine::general_purpose::URL_SAFE};
use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::Sha256;
use std::collections::HashMap;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostOrder {
    pub order: SignedOrderRequest,
    pub owner: String,
    pub order_type: OrderType,
    pub defer_exec: bool,
}

impl PostOrder {
    pub fn new(
        order: SignedOrderRequest,
        owner: String,
        order_type: OrderType,
        defer_exec: bool,
    ) -> Self {
        Self { order, owner, order_type, defer_exec }
    }
}

#[derive(Clone)]
pub struct TradingClient {
    api_base: String,
    client: reqwest::Client,
    wallet: PrivateKeySigner,
    creds: Credentials,
}

impl TradingClient {
    pub fn new(api_base: &str, wallet: PrivateKeySigner, creds: Credentials) -> Result<Self> {
        let client = reqwest::Client::builder().build()?;
        Ok(Self { api_base: api_base.to_string(), client, wallet, creds })
    }

    pub async fn post_order(
        &self,
        order: SignedOrderRequest,
        order_type: OrderType,
    ) -> Result<serde_json::Value> {
        let post_order = PostOrder::new(order, self.creds.api_key.clone(), order_type, false);
        let headers =
            create_l2_headers(&self.wallet, &self.creds, "POST", "/order", Some(&post_order))?;
        let url = format!("{}{}", self.api_base, "/order");
        let mut request = self.client.post(&url).json(&post_order);
        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request.send().await?;
        into_result(response).await
    }

    pub async fn ok(&self) -> Result<serde_json::Value> {
        let url = format!("{}{}", self.api_base, "/ok");

        let response = self.client.get(&url).send().await?;
        into_result(response).await
    }
}

pub fn create_l2_headers<T>(
    signer: &PrivateKeySigner,
    api_creds: &Credentials,
    method: &str,
    req_path: &str,
    body: Option<&T>,
) -> Result<HashMap<&'static str, String>>
where
    T: ?Sized + Serialize,
{
    let address = encode_prefixed(signer.address().as_slice());
    let timestamp = get_current_unix_time_secs();

    let hmac_signature =
        build_hmac_signature(&api_creds.secret, timestamp, method, req_path, body)?;

    Ok(HashMap::from([
        (POLY_ADDR_HEADER, address),
        (POLY_SIG_HEADER, hmac_signature),
        (POLY_TS_HEADER, timestamp.to_string()),
        (POLY_API_KEY_HEADER, api_creds.api_key.clone()),
        (POLY_PASS_HEADER, api_creds.passphrase.clone()),
    ]))
}

/// Builds an HMAC-SHA256 signature for Polymarket's derived API key authentication.
///
/// This is the standard signing method used after calling `derive-api-key`.
/// The signature string is constructed as:
///
/// ```text
/// {timestamp}{http_method}{request_path}{compact_json_body}
/// ```
///
/// - If `body` is `None` (e.g. GET/DELETE requests), the body part is omitted.
/// - If `body` is present, it is serialized to compact JSON (no whitespace).
///
/// The HMAC is computed using SHA-256 with the **base64-url-decoded** API secret as the key,
/// and the final digest is base64-url-encoded (no padding).
///
/// This exact format is required by `POLY_SIGNATURE` header when using API key + passphrase auth.
pub fn build_hmac_signature<T>(
    secret: &str,
    timestamp: u64,
    method: &str,
    req_path: &str,
    body: Option<&T>,
) -> Result<String, anyhow::Error>
where
    T: ?Sized + Serialize,
{
    // Decode the base64-url-encoded secret key
    let decoded =
        URL_SAFE.decode(secret).map_err(|e| anyhow::anyhow!("Failed to decode secret: {e}"))?;

    // Build the pre-image message exactly as the Polymarket backend expects
    let message = match body {
        None => format!("{timestamp}{method}{req_path}"),
        Some(b) => {
            // Compact JSON serialization (no pretty-printing or spaces)
            let serialized = serde_json::to_string(b)?;
            format!("{timestamp}{method}{req_path}{serialized}")
        }
    };

    // Initialize HMAC-SHA256 with the decoded secret
    let mut mac = HmacSha256::new_from_slice(&decoded)
        .map_err(|e| anyhow::anyhow!("HMAC initialization error: {e}"))?;

    // Update with the message bytes
    mac.update(message.as_bytes());

    // Finalize and encode the digest in base64-url format
    let result = mac.finalize();
    Ok(URL_SAFE.encode(result.into_bytes()))
}
