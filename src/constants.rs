// binary strings
pub const B_CYPHER_GROUP: &[u8] = b"cypher_group";
pub const B_CYPHER_USER: &[u8] = b"cypher_user";
pub const B_DEX_MARKET_AUTHORITY: &[u8] = b"dex_market_authority";
pub const B_OPEN_ORDERS: &[u8] = b"open_orders";

// group
pub const MARKETS_MAX_CNT: usize = 15;
pub const TOKENS_MAX_CNT: usize = MARKETS_MAX_CNT + 1;
pub const QUOTE_TOKEN_IDX: usize = TOKENS_MAX_CNT - 1;
