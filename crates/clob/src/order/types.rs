use std::str::FromStr;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderSide {
    #[serde(rename = "BUY")]
    Buy,
    #[serde(rename = "SELL")]
    Sell,
}

impl OrderSide {
    /// Convert side to numeric value (0 for BUY, 1 for SELL)
    pub fn to_u8(self) -> u8 {
        match self {
            OrderSide::Buy => 0,
            OrderSide::Sell => 1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RoundConfig {
    pub price: u32,
    pub size: u32,
    pub amount: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TickSize {
    /// 0.1
    Tenth,
    /// 0.01
    Hundredth,
    /// 0.001
    Thousandth,
    /// 0.0001
    TenThousandth,
}

impl TickSize {
    pub fn as_f64(self) -> f64 {
        match self {
            Self::Tenth => 0.1,
            Self::Hundredth => 0.01,
            Self::Thousandth => 0.001,
            Self::TenThousandth => 0.0001,
        }
    }

    pub fn round_config(self) -> RoundConfig {
        match self {
            TickSize::Tenth => RoundConfig { price: 1, size: 2, amount: 3 },
            TickSize::Hundredth => RoundConfig { price: 2, size: 2, amount: 4 },
            TickSize::Thousandth => RoundConfig { price: 3, size: 2, amount: 5 },
            TickSize::TenThousandth => RoundConfig { price: 4, size: 2, amount: 6 },
        }
    }
}

impl FromStr for TickSize {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0.1" => Ok(Self::Tenth),
            "0.01" => Ok(Self::Hundredth),
            "0.001" => Ok(Self::Thousandth),
            "0.0001" => Ok(Self::TenThousandth),
            _ => Err("invalid tick size"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OrderKind {
    /// Limit order with a fixed size in base token units  
    /// The user specifies the exact amount of outcome tokens (YES/NO shares) they want to buy or sell  
    /// at a given limit price (or better).  
    /// Example: "Buy 500 YES shares at max $0.65 each" → size = 500
    Limit {
        /// Quantity in base token units (outcome shares / conditional tokens), not in dollars
        size: Decimal,
    },

    /// Market buy order using a fixed quote (USDC) amount  
    /// The user specifies exactly how much USDC they are willing to spend.  
    /// The order will be filled immediately at the current best available ask price(s)  
    /// until the quote amount is exhausted.
    /// Example: "Buy as many YES shares as possible with exactly $1000 USDC"
    MarketBuy {
        /// Total amount in USDC (quote currency) to spend
        quote_amount: Decimal,
    },

    /// Market sell order using a fixed base token amount  
    /// The user specifies exactly how many outcome tokens (YES/NO shares) they want to sell.  
    /// The order will be filled immediately at the current best available bid price(s)  
    /// until the entire base amount is sold.
    /// Example: "Sell 2500 NO shares at market price"
    MarketSell {
        /// Quantity in outcome tokens (base token units) to sell
        base_amount: Decimal,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureType {
    /// EOA (Externally Owned Account)  
    /// Standard EIP-712 signature produced directly by a private key belonging to an externally owned account.  
    /// This is the default and most common signature type.
    #[serde(rename = "0")]
    Eoa = 0,

    /// Polymarket Proxy Wallet  
    /// EIP-712 signature generated using the signer associated with a user's funded Polymarket Proxy wallet.  
    /// Allows trading without moving funds out of the official Polymarket smart wallet infrastructure.
    #[serde(rename = "1")]
    PolyProxy = 1,

    /// Polymarket Gnosis Safe Wallet  
    /// EIP-712 signature generated using the signer associated with a user's funded Polymarket Gnosis Safe.  
    /// Enables institutional or multi-sig users to trade directly via their Safe while keeping funds in the official system.  
    /// (Currently supported in the API but not yet widely used in production.)
    #[serde(rename = "2")]
    PolyGnosisSafe = 2,
}

impl SignatureType {
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(SignatureType::Eoa),
            1 => Some(SignatureType::PolyProxy),
            2 => Some(SignatureType::PolyGnosisSafe),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedOrderRequest {
    pub salt: u64,
    pub maker: String,
    pub signer: String,
    pub taker: String,
    #[serde(rename = "tokenId")]
    pub token_id: String,
    #[serde(rename = "makerAmount")]
    pub maker_amount: String,
    #[serde(rename = "takerAmount")]
    pub taker_amount: String,
    pub expiration: String,
    pub nonce: String,
    #[serde(rename = "feeRateBps")]
    pub fee_rate_bps: String,
    pub side: OrderSide,
    #[serde(rename = "signatureType")]
    pub signature_type: u8,
    pub signature: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderType {
    /// Good-Til-Cancelled  
    /// A limit order that remains active until it is fully filled or explicitly cancelled by the user.
    #[serde(rename = "GTC")]
    Gtc,

    /// Fill-Or-Kill  
    /// An order that must be executed immediately and in its entirety at the specified price (or better).  
    /// If the full quantity cannot be filled instantly, the entire order is cancelled.
    #[serde(rename = "FOK")]
    Fok,

    /// Fill-And-Kill (a.k.a. Immediate-Or-Cancel)  
    /// An order that is executed immediately for as much quantity as possible at the specified price (or better).  
    /// Any remaining unfilled portion is cancelled immediately.
    #[serde(rename = "FAK")]
    Fak,

    /// Good-Til-Date  
    /// A limit order that remains active until either it is filled, cancelled, or the specified expiration timestamp (Unix seconds, UTC) is reached.  
    /// Note: Polymarket enforces a minimum expiration buffer of 1 minute. If the desired expiration is less than 90 seconds away,  
    /// the actual expiration must be set to `current_time + 90 seconds`.
    #[serde(rename = "GTD")]
    Gtd,
}
