pub mod client;
pub mod constants;

use anchor_lang::prelude::*;
use constants::*;
use jet_proto_math::Number;
use std::mem::take;

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
    pub fn iter_positions<'a>(&'a self) -> impl Iterator<Item = &UserPosition> {
        struct Iter<'a> {
            positions: &'a [UserPosition],
        }
        impl<'a> Iterator for Iter<'a> {
            type Item = &'a UserPosition;
            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    match take(&mut self.positions).split_first() {
                        Some((head, rems)) => {
                            self.positions = rems;
                            if head.market_idx < MARKETS_MAX_CNT as u8 {
                                return Some(head);
                            }
                        }
                        None => return None,
                    }
                }
            }
        }
        Iter {
            positions: &self.positions[..],
        }
    }

    fn get_position_idx(&self, token_idx: usize) -> Option<usize> {
        if token_idx == QUOTE_TOKEN_IDX {
            Some(QUOTE_TOKEN_IDX)
        } else {
            self.positions
                .iter()
                .position(|p| (p.market_idx as usize) == token_idx)
        }
    }

    /// gets the user's position for the given market index
    pub fn get_position(&self, token_idx: usize) -> Option<&UserPosition> {
        let idx = self.get_position_idx(token_idx);
        if let Some(idx) = idx {
            self.positions.get(idx)
        } else {
            None
        }
    }

    /// gets the users's assets value
    pub fn get_assets_value(&self, group: &CypherGroup) -> Number {
        let quote_token = group.get_cypher_token(QUOTE_TOKEN_IDX).unwrap();
        let quote_position = self.get_position(QUOTE_TOKEN_IDX);
        let quote_deposits = if let Some(position) = quote_position {
            position.total_deposits(quote_token)
        } else {
            Number::ZERO
        };
        let mut assets_value = quote_deposits;

        for position in self.iter_positions() {
            let market_idx = position.market_idx as usize;
            let market = group.get_cypher_market(market_idx);
            let market_price = if let Some(m) = market {
                m.market_price
            } else {
                continue;
            };
            let oo_info = &position.oo_info;
            if oo_info.is_account_open {
                let oo_coin_value = oo_info.coin_total * market_price;
                let oo_value = oo_coin_value + oo_info.pc_total + oo_info.referrer_rebates_accrued;
                assets_value += oo_value.into();
            }
            assets_value += position.base_deposits() * market_price;
        }
        assets_value
    }

    /// gets the users's liabilities value
    pub fn get_liabilities_value(&self, group: &CypherGroup) -> Number {
        let quote_token = group.get_cypher_token(QUOTE_TOKEN_IDX).unwrap();
        let quote_position = self.get_position(QUOTE_TOKEN_IDX);
        let quote_borrows = if let Some(position) = quote_position {
            position.total_borrows(quote_token)
        } else {
            Number::ZERO
        };
        let mut liabs_value = quote_borrows;

        for position in self.iter_positions() {
            let market_idx = position.market_idx as usize;
            let market = group.get_cypher_market(market_idx);
            let market_price = if let Some(m) = market {
                m.market_price
            } else {
                continue;
            };
            liabs_value += position.base_borrows() * market_price;
        }
        liabs_value
    }

    /// gets the user's margin c-ratio
    pub fn get_margin_c_ratio(&self, group: &CypherGroup) -> Number {
        let liabs_value = self.get_liabilities_value(group);
        if liabs_value == Number::ZERO {
            Number::MAX
        } else {
            let assets_value = self.get_assets_value(group);
            assets_value / liabs_value
        }
    }

    /// gets the user's margin c-ratio components
    /// the first number is the margin c-ratio, the second number is the assets value and the third  number is the liabilites value
    pub fn get_margin_c_ratio_components(&self, group: &CypherGroup) -> (Number, Number, Number) {
        let liabs_value = self.get_liabilities_value(group);
        if liabs_value == Number::ZERO {
            (Number::MAX, self.get_assets_value(group), liabs_value)
        } else {
            let assets_value = self.get_assets_value(group);
            (assets_value / liabs_value, assets_value, liabs_value)
        }
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
