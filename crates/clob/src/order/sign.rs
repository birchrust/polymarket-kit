use crate::{POLYGON_MAINNET_CHAIN_ID, get_current_unix_time_secs};
use alloy_primitives::{Address, hex::encode_prefixed};
use alloy_signer::SignerSync;
use alloy_signer_local::PrivateKeySigner;
use alloy_sol_types::SolStruct;
use alloy_sol_types::{eip712_domain, sol};
use anyhow::{Error, Result};
use rand::{Rng, rng};

sol! {
    struct Order {
        uint256 salt;
        address maker;
        address signer;
        address taker;
        uint256 tokenId;
        uint256 makerAmount;
        uint256 takerAmount;
        uint256 expiration;
        uint256 nonce;
        uint256 feeRateBps;
        uint8 side;
        uint8 signatureType;
    }
}

pub fn generate_seed() -> Result<u64> {
    let mut rng = rng();
    let y: f64 = rng.random();
    let timestamp = get_current_unix_time_secs();
    let a: f64 = timestamp as f64 * y;
    Ok(a as u64)
}

pub fn sign_order_message(
    signer: &PrivateKeySigner,
    order: Order,
    verifying_contract: Address,
) -> Result<String> {
    let domain = eip712_domain!(
        name: "Polymarket CTF Exchange",
        version: "1",
        chain_id: POLYGON_MAINNET_CHAIN_ID,
        verifying_contract: verifying_contract,
    );

    let hash = order.eip712_signing_hash(&domain);
    let signature = signer
        .sign_hash_sync(&hash)
        .map_err(|e| Error::msg(format!("Failed to sign order: {e}")))?;

    Ok(encode_prefixed(signature.as_bytes()))
}
