use alloy_primitives::{Address, address};
use rust_decimal::Decimal;

/// WebSocket endpoint for RTSD (Real-Time Streaming Data)
pub const RTSD_WEBSOCKET_URL: &str = "wss://ws-live-data.polymarket.com";

/// Polymarket market-specific WebSocket subscription endpoint
pub const POLYMARKET_MARKET_WS_URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";

/// Base URL for Gamma API (public market data, prices, order books, etc.)
pub const GAMMA_API_URL: &str = "https://gamma-api.polymarket.com";

/// Base URL for CLOB (Central Limit Order Book) API – used for trading/auth
pub const CLOB_API_URL: &str = "https://clob.polymarket.com";

/// Base URL for Polymarket Data API (historical data, events, etc.)
pub const DATA_API_URL: &str = "https://data-api.polymarket.com/";

/// Polygon Mainnet chain ID
pub const POLYGON_MAINNET_CHAIN_ID: u64 = 137;

/// USDCe token contract on Polygon (6 decimals)
pub const POLYGON_COLLATERAL_CONTRACT: Address =
    address!("2791bca1f2de4661ed88a30c99a7a9449aa84174");

/// Conditional Tokens framework contract (CTF) on Polygon
pub const POLYGON_CONDITIONAL_TOKEN_CONTRACT: Address =
    address!("4D97DCd97eC945f40cF65F87097ACe5EA0476045");

/// Neg-Risk (negative risk) exchange adapter contract
pub const POLYGON_NEG_RISK_EXCHANGE_CONTRACT: Address =
    address!("C5d563A36AE78145C45a50134d48A1215220f80a");

/// Main Polymarket exchange contract (yes/no markets)
pub const POLYGON_EXCHANGE_CONTRACT: Address = address!("4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E");

/// Scaling factor for token amounts (USDC has 6 decimals → 1_000_000)
pub const TOKEN_SCALE: Decimal = Decimal::from_parts(1_000_000, 0, 0, false, 0);

/// Header containing the wallet address for Polymarket signed requests
pub const POLY_ADDR_HEADER: &str = "POLY_ADDRESS";

/// Header containing the EIP-712 signature
pub const POLY_SIG_HEADER: &str = "POLY_SIGNATURE";

/// Header containing the request timestamp (Unix seconds)
pub const POLY_TS_HEADER: &str = "POLY_TIMESTAMP";

/// Header containing the nonce (prevents replay attacks)
pub const POLY_NONCE_HEADER: &str = "POLY_NONCE";

/// Header for API key (used after derive-api-key flow)
pub const POLY_API_KEY_HEADER: &str = "POLY_API_KEY";

/// Header for passphrase (part of derived credentials)
pub const POLY_PASS_HEADER: &str = "POLY_PASSPHRASE";
