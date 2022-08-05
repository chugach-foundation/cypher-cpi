pub mod client;

use anchor_lang::prelude::*;
use jet_proto_math::Number;

anchor_gen::generate_cpi_interface!(
    idl_path = "idl.json",
    zero_copy(
        PriceHistory,
        CypherGroup,
        CypherGroupConfig,
        CypherMarket,
        CypherMarketConfig,
        CypherToken,
        CypherUser,
        UserPosition,
        OpenOrdersInfo,
        PriceHistory
    )
);

#[cfg(feature = "mainnet-beta")]
declare_id!("CYPHER79cJLzQ8iyyr6oeizfGgR9YU9NM9oTMPWak5oQ");
#[cfg(not(feature = "mainnet-beta"))]
declare_id!("8Z8nDAa98hgdYCS9SyAyAesxE3ZhAq8Qo1E8v2V8VU56");

pub mod devnet_faucet {
    use anchor_lang::declare_id;

    #[cfg(feature = "devnet")]
    declare_id!("7njrvFJx4NJQvzywv1LdnPwzYYTSh1wWgGL5vkwTUuSS");
    #[cfg(not(feature = "devnet"))]
    declare_id!("7njrvFJx4NJQvzywv1LdnPwzYYTSh1wWgGL5vkwTUuSS");
}

pub mod quote_mint {
    use anchor_lang::declare_id;
    #[cfg(feature = "mainnet-beta")]
    declare_id!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    #[cfg(not(feature = "mainnet-beta"))]
    declare_id!("DPhNUKVhnrkdbq37GUgTUBRbZLsvziX1p5e5YUXyjBsb");
}

// group
pub const MARKETS_MAX_CNT: usize = 15;
pub const TOKENS_MAX_CNT: usize = MARKETS_MAX_CNT + 1;
pub const QUOTE_TOKEN_IDX: usize = TOKENS_MAX_CNT - 1;

impl CypherGroup {
    /// gets the group's margin initialization ratio
    pub fn margin_init_ratio(&self) -> Number {
        Number::from_percent(self.config.margin_init_ratio)
    }

    /// gets the group's margin maintenance ratio
    pub fn margin_maint_ratio(&self) -> Number {
        Number::from_percent(self.config.margin_maint_ratio)
    }

    /// gets the group's partial margin ratio, this is used as target ratio during liquidations
    pub fn margin_partial_ratio(&self) -> Number {
        Number::from_percent(self.config.margin_partial_ratio)
    }

    /// gets the group's liquidator bonus fee
    pub fn liq_liqor_fee(&self) -> Number {
        Number::ONE + Number::from_bps(self.config.liquidator_bonus_bps)
    }

    /// gets the group's liquidation insurance fee
    pub fn liq_insurance_fee(&self) -> Number {
        Number::from_bps(self.config.liquidation_insurance_fee_bps)
    }

    /// gets the cypher token at the given index
    pub fn get_cypher_token(&self, token_index: usize) -> Option<&CypherToken> {
        if self.tokens[token_index].mint == Pubkey::default() {
            return None;
        }
        self.tokens.get(token_index)
    }

    /// gets the cypher market at the given index
    pub fn get_cypher_market(&self, market_index: usize) -> Option<&CypherMarket> {
        if self.tokens[market_index].mint == Pubkey::default() {
            return None;
        }
        self.markets.get(market_index)
    }

    /// gets index of the token with the given `c_asset_mint`
    pub fn get_token_idx(&self, mint: Pubkey) -> Option<usize> {
        self.tokens
            .iter()
            .filter(|t| t.mint != Pubkey::default())
            .position(|token| token.mint == mint)
    }

    /// gets index of the market with the given `c_asset_mint`
    ///
    /// if the given `c_asset_mint` is the same as the `quote_mint::ID`, returns `None`
    pub fn get_market_idx(&self, c_asset_mint: Pubkey) -> Option<usize> {
        if c_asset_mint == quote_mint::ID {
            return None;
        }
        self.get_token_idx(c_asset_mint)
    }

    /// gets the quote token vault pubkey
    pub fn quote_vault(&self) -> Pubkey {
        self.tokens[QUOTE_TOKEN_IDX].vault
    }
}

impl CypherToken {
    /// checks whether the token is the quote token
    pub fn is_quote(&self) -> bool {
        self.mint == quote_mint::ID
    }

    /// gets the token's decimals
    pub fn decimals(&self) -> u8 {
        self.config.decimals
    }

    /// gets the token's deposit index
    pub fn deposit_index(&self) -> Number {
        Number::from_bytes(self.deposit_index)
    }

    /// gets the token's borrow index
    pub fn borrow_index(&self) -> Number {
        Number::from_bytes(self.borrow_index)
    }

    /// gets the base deposit amount
    pub fn base_deposits(&self) -> Number {
        Number::from_bytes(self.base_deposits)
    }

    /// gets the base borrows amount
    pub fn base_borrows(&self) -> Number {
        Number::from_bytes(self.base_borrows)
    }

    /// gets the total deposits adjusted for the token's deposit index
    pub fn total_deposits(&self) -> Number {
        self.base_deposits() * self.deposit_index()
    }

    /// gets the total borrows adjusted for the token's borrow index
    pub fn total_borrows(&self) -> Number {
        self.base_borrows() * self.borrow_index()
    }
}

impl CypherMarket {
    /// gets the latest cached oracle price
    pub fn oracle_price(&self) -> u64 {
        self.oracle_price.price
    }
}

impl CypherUser {
    /// gets the user's position for the given market index
    pub fn get_user_position(&self, market_index: usize) -> Option<&UserPosition> {
        if self.positions[market_index].market_idx == u8::default() {
            return None;
        }
        self.positions.get(market_index)
    }
}

impl UserPosition {    
    /// gets the base deposit amount
    pub fn base_deposits(&self) -> Number {
        Number::from_bytes(self.base_deposits)
    }

    /// gets the base borrows amount
    pub fn base_borrows(&self) -> Number {
        Number::from_bytes(self.base_borrows)
    }

    /// gets the user's total deposits adjusted for the token's deposit index
    pub fn total_deposits(&self, cypher_token: &CypherToken) -> Number {
        self.base_deposits() * cypher_token.deposit_index()
    }
    
    /// gets the user's total borrows adjusted for the token's borrow index
    pub fn total_borrows(&self, cypher_token: &CypherToken) -> Number {
        self.base_borrows() * cypher_token.borrow_index()
    }
}