use std::collections::HashMap;

use crate::{
    POLY_ADDR_HEADER, POLY_NONCE_HEADER, POLY_SIG_HEADER, POLY_TS_HEADER, POLYGON_MAINNET_CHAIN_ID,
    get_current_unix_time_secs, into_result,
};
use alloy_primitives::{U256, hex::encode_prefixed};
use alloy_signer::SignerSync;
use alloy_signer_local::PrivateKeySigner;
use alloy_sol_types::SolStruct;
use alloy_sol_types::{eip712_domain, sol};
use anyhow::{Error, Result};
use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Credentials {
    #[serde(rename = "apiKey")]
    pub api_key: String,
    pub secret: String,
    pub passphrase: String,
}

impl Credentials {
    pub fn new(api_key: String, secret: String, passphrase: String) -> Self {
        Self { api_key, secret, passphrase }
    }
}

pub struct AuthenticatedClient {
    api_base: String,
    client: reqwest::Client,
    wallet: PrivateKeySigner,
}

impl AuthenticatedClient {
    pub fn new(api_base: &str, wallet: PrivateKeySigner) -> Result<Self> {
        let client = reqwest::Client::builder().build()?;
        Ok(Self { api_base: api_base.to_string(), client, wallet })
    }

    fn auth_request(&self, builder: RequestBuilder) -> Result<RequestBuilder> {
        let headers = create_l1_headers(&self.wallet, POLYGON_MAINNET_CHAIN_ID, None)?;
        let mut req = builder;
        for (k, v) in headers {
            req = req.header(k, v);
        }
        Ok(req)
    }

    pub async fn derive_api_key(&self) -> Result<Credentials> {
        let url = format!("{}/auth/derive-api-key", self.api_base);
        let request = self.client.get(&url);
        let request = self.auth_request(request)?;

        let response = request.send().await?;
        into_result(response).await
    }
}

pub fn create_l1_headers(
    signer: &PrivateKeySigner,
    chain_id: u64,
    nonce: Option<U256>,
) -> Result<HashMap<&'static str, String>> {
    let timestamp = get_current_unix_time_secs().to_string();
    let nonce_val = nonce.unwrap_or(U256::ZERO);
    let signature = sign_clob_auth_message(signer, timestamp.clone(), nonce_val, chain_id)?;
    let address = encode_prefixed(signer.address().as_slice());

    Ok(HashMap::from([
        (POLY_ADDR_HEADER, address),
        (POLY_SIG_HEADER, signature),
        (POLY_TS_HEADER, timestamp),
        (POLY_NONCE_HEADER, nonce_val.to_string()),
    ]))
}

sol! {
    struct ClobAuth {
        address address;
        string timestamp;
        uint256 nonce;
        string message;
    }
}

pub fn sign_clob_auth_message(
    signer: &PrivateKeySigner,
    timestamp: String,
    nonce: U256,
    chain_id: u64,
) -> Result<String> {
    let message = "This message attests that I control the given wallet".to_owned();

    let auth_struct = ClobAuth { address: signer.address(), timestamp, nonce, message };

    let domain = eip712_domain!(
        name: "ClobAuthDomain",
        version: "1",
        chain_id: chain_id,
    );

    let hash = auth_struct.eip712_signing_hash(&domain);
    let signature = signer
        .sign_hash_sync(&hash)
        .map_err(|e| Error::msg(format!("Failed to sign auth message: {e}")))?;

    Ok(encode_prefixed(signature.as_bytes()))
}
