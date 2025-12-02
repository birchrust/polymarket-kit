use crate::{Order, OrderKind, SignatureType, SignedOrderRequest, TOKEN_SCALE, generate_seed};
use crate::{OrderSide, RoundConfig, TickSize};
use crate::{POLYGON_EXCHANGE_CONTRACT, POLYGON_NEG_RISK_EXCHANGE_CONTRACT, sign_order_message};
use alloy_primitives::{Address, U256};
use alloy_signer_local::PrivateKeySigner;
use anyhow::{Error, Result};
use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy::AwayFromZero;
use rust_decimal::RoundingStrategy::MidpointTowardZero;
use rust_decimal::RoundingStrategy::ToZero;
use std::str::FromStr;

pub struct OrderParams {
    pub token_id: String,
    pub price: Decimal,
    pub side: OrderSide,
    pub nonce: Option<U256>,
    pub fee_rate_bps: Option<u32>,
    pub expiration: Option<u64>,
    pub taker: Option<Address>,
    pub signer: Address,
    pub funder: Option<Address>,
    pub tick_size: String,
    pub kind: OrderKind,
    pub sig_type: SignatureType,
    pub neg_risk: bool,
    pub wallet: PrivateKeySigner,
}

pub async fn create_order(params: OrderParams) -> Result<SignedOrderRequest, Error> {
    let signer = params.wallet.address();
    let nonce = params.nonce.unwrap_or(U256::ZERO);
    let fee_rate_bps = params.fee_rate_bps.unwrap_or(0_u32);
    let expiration = params.expiration.unwrap_or(0_u64);
    let taker = params.taker.unwrap_or(Address::ZERO);
    let funder = params.funder.unwrap_or(signer);
    let tick_size = TickSize::from_str(&params.tick_size).map_err(|e| Error::msg(e))?;

    let (maker_amount, taker_amount) =
        calculate_order_amounts(params.price, params.side, params.kind, tick_size);

    let seed = generate_seed()?;

    let u256_token_id = U256::from_str_radix(&params.token_id, 10)
        .map_err(|e| Error::msg(format!("Invalid token_id: {}", e)))?;

    let salt = U256::from(seed);

    let order = Order {
        salt,
        maker: funder,
        signer,
        taker,
        tokenId: u256_token_id,
        makerAmount: U256::from(maker_amount),
        takerAmount: U256::from(taker_amount),
        expiration: U256::from(expiration),
        nonce,
        feeRateBps: U256::from(fee_rate_bps),
        side: params.side.to_u8(),
        signatureType: params.sig_type.to_u8(),
    };

    let exchange_contract = match params.neg_risk {
        true => POLYGON_NEG_RISK_EXCHANGE_CONTRACT,
        false => POLYGON_EXCHANGE_CONTRACT,
    };
    let signature = sign_order_message(&params.wallet, order, exchange_contract)?;

    Ok(SignedOrderRequest {
        salt: seed,
        maker: funder.to_string(),
        signer: signer.to_string(),
        taker: taker.to_string(),
        token_id: params.token_id,
        maker_amount: maker_amount.to_string(),
        taker_amount: taker_amount.to_string(),
        expiration: expiration.to_string(),
        nonce: nonce.to_string(),
        fee_rate_bps: fee_rate_bps.to_string(),
        side: params.side,
        signature_type: params.sig_type.to_u8(),
        signature,
    })
}

#[inline]
/// Calculates the final **maker** and **taker** token amounts required by the Polymarket CLOB
/// from a user-facing order specification.
///
/// The CLOB always expects amounts in **whole token units (u32)** and applies strict
/// rounding rules defined by the market's `TickSize`. This function performs all the
/// required rounding and conversion steps so the resulting values can be sent directly
/// in an order payload.
///
/// Returns `(maker_amount, taker_amount)`:
/// - `maker_amount` – the amount of the **maker** token (what you give)
/// - `taker_amount` – the amount of the **taker** token (what you receive)
///
/// The maker/taker assignment follows Polymarket's convention:
/// - **Buy** side:  maker = USDC (quote), taker = outcome shares (base)
/// - **Sell** side: maker = outcome shares (base), taker = USDC (quote)
pub fn calculate_order_amounts(
    price: Decimal,
    side: OrderSide,
    kind: OrderKind,
    tick_size: TickSize,
) -> (u32, u32) {
    let round_cfg = tick_size.round_config();

    // Price must be rounded to tick precision first (shared by all cases)
    let raw_price = price.round_dp_with_strategy(round_cfg.price, MidpointTowardZero);

    match (kind, side) {
        // ── Limit Buy ─────────────────────────────────────────────────────
        // User specifies exact base size (outcome shares) they want to buy.
        // maker = USDC (quote), taker = shares (base)
        (OrderKind::Limit { size }, OrderSide::Buy) => {
            let raw_taker_amt = size.round_dp_with_strategy(round_cfg.size, ToZero); // base shares
            let raw_maker_amt = fix_amount_rounding(raw_taker_amt * raw_price, &round_cfg); // USDC

            (
                decimal_to_token_u32(raw_maker_amt), // maker: USDC to spend
                decimal_to_token_u32(raw_taker_amt), // taker: shares to receive
            )
        }

        // ── Limit Sell ────────────────────────────────────────────────────
        // User specifies exact base size (outcome shares) they want to sell.
        // maker = shares (base), taker = USDC (quote)
        (OrderKind::Limit { size }, OrderSide::Sell) => {
            let raw_maker_amt = size.round_dp_with_strategy(round_cfg.size, ToZero); // base shares
            let raw_taker_amt = fix_amount_rounding(raw_maker_amt * raw_price, &round_cfg); // USDC

            (
                decimal_to_token_u32(raw_maker_amt), // maker: shares to give
                decimal_to_token_u32(raw_taker_amt), // taker: USDC to receive
            )
        }

        // ── Market Buy ────────────────────────────────────────────────────
        // User specifies exact USDC amount they want to spend.
        // maker = USDC (quote), taker = shares (base)
        (OrderKind::MarketBuy { quote_amount }, OrderSide::Buy) => {
            let raw_quote = quote_amount.round_dp_with_strategy(round_cfg.size, ToZero); // USDC
            let raw_base = fix_amount_rounding(raw_quote / raw_price, &round_cfg); // shares

            (
                decimal_to_token_u32(raw_quote), // maker: USDC to spend
                decimal_to_token_u32(raw_base),  // taker: shares to receive
            )
        }

        // ── Market Sell ───────────────────────────────────────────────────
        // User specifies exact amount of outcome shares they want to sell.
        // maker = shares (base), taker = USDC (quote)
        (OrderKind::MarketSell { base_amount }, OrderSide::Sell) => {
            let raw_base = base_amount.round_dp_with_strategy(round_cfg.size, ToZero); // shares
            let raw_quote = fix_amount_rounding(raw_base * raw_price, &round_cfg); // USDC

            (
                decimal_to_token_u32(raw_base),  // maker: shares to give
                decimal_to_token_u32(raw_quote), // taker: USDC to receive
            )
        }

        // Defensive fallback – should never happen with proper validation
        _ => (0, 0),
    }
}

#[inline]
fn fix_amount_rounding(mut amt: Decimal, round_config: &RoundConfig) -> Decimal {
    if amt.scale() > round_config.amount {
        amt = amt.round_dp_with_strategy(round_config.amount + 4, AwayFromZero);

        if amt.scale() > round_config.amount {
            amt = amt.round_dp_with_strategy(round_config.amount, ToZero);
        }
    }
    amt
}

#[inline]
fn decimal_to_token_u32(amt: Decimal) -> u32 {
    let mut scaled = TOKEN_SCALE * amt;
    if scaled.scale() > 0 {
        scaled = scaled.round_dp_with_strategy(0, MidpointTowardZero);
    }
    scaled.try_into().expect("Couldn't round decimal to u32 token units")
}
