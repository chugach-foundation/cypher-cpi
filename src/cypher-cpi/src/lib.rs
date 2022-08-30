pub mod client;
pub mod constants;
pub mod serum_cpi;
pub mod serum_slab;
pub mod utils;

use anchor_lang::prelude::*;
use constants::*;
use fixed::types::I80F48;
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
        DerivativesMarketInfo,
        SpotMarketInfo,
        LiquidityMiningInfo,
        OraclePrice,
        CypherUser,
        UserPosition,
        OpenOrdersInfo,
        PriceHistory
    ),
    
);

#[cfg(feature = "mainnet-beta")]
declare_id!("CYPHER3ziDd1rasgBcGGbx4fMtSS72x6NEM5Zvx2vNmK");
#[cfg(not(feature = "mainnet-beta"))]
declare_id!("cyph3iWWJctHgNosbRqxg4GjMHsEL8wAPBnKzPRxEdF");

pub mod quote_mint {
    use anchor_lang::declare_id;
    #[cfg(feature = "mainnet-beta")]
    declare_id!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    #[cfg(not(feature = "mainnet-beta"))]
    declare_id!("DPhNUKVhnrkdbq37GUgTUBRbZLsvziX1p5e5YUXyjBsb");
}

impl CypherGroup {
    /// gets the group's margin initialization ratio
    pub fn margin_init_ratio(&self) -> I80F48 {
        I80F48::from(self.config.margin_init_ratio)
    }

    /// gets the group's margin maintenance ratio
    pub fn margin_maint_ratio(&self) -> I80F48 {
        I80F48::from(self.config.margin_maint_ratio)
    }

    /// gets the group's partial margin ratio, this is used as target ratio during liquidations
    pub fn margin_partial_ratio(&self) -> I80F48 {
        I80F48::from(self.config.margin_partial_ratio)
    }

    /// gets the group's liquidator bonus fee
    pub fn liq_liqor_fee(&self) -> I80F48 {
        I80F48::ONE + I80F48::from(self.config.liquidator_bonus_bps)
    }

    /// gets the group's liquidation insurance fee
    pub fn liq_insurance_fee(&self) -> I80F48 {
        I80F48::from(self.config.liquidation_insurance_fee_bps)
    }

    /// gets the cypher token at the given index
    pub fn get_cypher_token(&self, token_index: usize) -> Option<&CypherToken> {
        if self.tokens[token_index].spot_mint == Pubkey::default()
            && self.tokens[token_index].c_asset_mint == Pubkey::default()
        {
            return None;
        }
        self.tokens.get(token_index)
    }

    /// gets the cypher market at the given index
    pub fn get_cypher_market(&self, market_index: usize) -> Option<&CypherMarket> {
        if self.markets[market_index].derivative_info.dex_market == Pubkey::default()
            && self.markets[market_index].spot_info.spot_market == Pubkey::default()
        {
            return None;
        }
        self.markets.get(market_index)
    }

    /// gets index of the token with the given `c_asset_mint`
    pub fn get_token_idx(&self, c_sset_mint: Pubkey) -> Option<usize> {
        self.tokens
            .iter()
            .filter(|t| t.c_asset_mint != Pubkey::default() || t.spot_mint != Pubkey::default())
            .position(|token| token.c_asset_mint == c_sset_mint)
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
        self.tokens[QUOTE_TOKEN_IDX].spot_vault
    }
}

impl CypherToken {
    /// checks whether the token is the quote token
    pub fn is_quote(&self) -> bool {
        self.spot_mint == quote_mint::ID
    }

    /// gets the token's decimals
    pub fn decimals(&self) -> u8 {
        self.config.decimals
    }

    /// gets the token's deposit index
    pub fn deposit_index(&self) -> I80F48 {
        I80F48::from_bits(self.deposit_index)
    }

    /// gets the token's borrow index
    pub fn borrow_index(&self) -> I80F48 {
        I80F48::from_bits(self.borrow_index)
    }

    /// gets the spot base deposit amount
    pub fn spot_base_deposits(&self) -> I80F48 {
        I80F48::from_bits(self.spot_base_deposits)
    }

    /// gets the spot base borrows amount
    pub fn spot_base_borrows(&self) -> I80F48 {
        I80F48::from_bits(self.spot_base_borrows)
    }

    /// gets the derivative base deposit amount
    pub fn deriv_base_deposits(&self) -> I80F48 {
        I80F48::from_bits(self.deriv_base_deposits)
    }

    /// gets the derivative base borrows amount
    pub fn deriv_base_borrows(&self) -> I80F48 {
        I80F48::from_bits(self.deriv_base_deposits)
    }

    /// gets the total deposits adjusted for the token's deposit index
    pub fn total_deposits(&self) -> I80F48 {
        self.spot_base_deposits() * self.deposit_index()
    }

    /// gets the total borrows adjusted for the token's borrow index
    pub fn total_borrows(&self) -> I80F48 {
        self.spot_base_borrows() * self.borrow_index()
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
    pub fn get_assets_value(&self, group: &CypherGroup) -> I80F48 {
        let quote_token = group.get_cypher_token(QUOTE_TOKEN_IDX).unwrap();
        let quote_position = self.get_position(QUOTE_TOKEN_IDX);
        let quote_deposits = if let Some(position) = quote_position {
            position.total_deposits(quote_token)
        } else {
            I80F48::ZERO
        };
        let mut assets_value = quote_deposits;

        for position in self.iter_positions() {
            let market_idx = position.market_idx as usize;
            let market = match group.get_cypher_market(market_idx) {
                Some(m) => m,
                None => {
                    continue;
                }
            };
            let market_price = I80F48::from(market.derivative_info.market_price);
            let oracle_price = I80F48::from(market.oracle_price.price);
            let deriv_oo_info = &position.deriv_oo_info;
            if deriv_oo_info.is_account_open {
                let oo_coin_value = I80F48::from(deriv_oo_info.coin_total) * market_price;
                let oo_value = oo_coin_value
                    + I80F48::from(deriv_oo_info.pc_total + deriv_oo_info.referrer_rebates_accrued);
                assets_value += oo_value;
            }
            let spot_oo_info = &position.spot_oo_info;
            if spot_oo_info.is_account_open {
                let oo_coin_value = I80F48::from(spot_oo_info.coin_total) * market_price;
                let oo_value = oo_coin_value
                    + I80F48::from(spot_oo_info.pc_total + spot_oo_info.referrer_rebates_accrued);
                assets_value += oo_value;
            }
            assets_value += position.deriv_base_deposits() * market_price;
            assets_value += position.spot_base_deposits() * oracle_price;
        }
        assets_value
    }

    /// gets the users's liabilities value
    pub fn get_liabilities_value(&self, group: &CypherGroup) -> I80F48 {
        let quote_token = group.get_cypher_token(QUOTE_TOKEN_IDX).unwrap();
        let quote_position = self.get_position(QUOTE_TOKEN_IDX);
        let quote_borrows = if let Some(position) = quote_position {
            position.total_borrows(quote_token)
        } else {
            I80F48::ZERO
        };
        let mut liabs_value = quote_borrows;

        for position in self.iter_positions() {
            let market_idx = position.market_idx as usize;
            let market = group.get_cypher_market(market_idx);
            let market_price = if let Some(m) = market {
                I80F48::from(m.derivative_info.market_price)
            } else {
                continue;
            };
            liabs_value += position.deriv_base_borrows() * market_price;
        }
        liabs_value
    }

    /// gets the user's margin c-ratio
    pub fn get_margin_c_ratio(&self, group: &CypherGroup) -> I80F48 {
        let liabs_value = self.get_liabilities_value(group);
        if liabs_value == I80F48::ZERO {
            I80F48::MAX
        } else {
            let assets_value = self.get_assets_value(group);
            assets_value / liabs_value
        }
    }

    /// gets the user's margin c-ratio components
    /// the first number is the margin c-ratio, the second number is the assets value and the third  number is the liabilites value
    pub fn get_margin_c_ratio_components(&self, group: &CypherGroup) -> (I80F48, I80F48, I80F48) {
        let liabs_value = self.get_liabilities_value(group);
        if liabs_value == I80F48::ZERO {
            (I80F48::MAX, self.get_assets_value(group), liabs_value)
        } else {
            let assets_value = self.get_assets_value(group);
            (assets_value / liabs_value, assets_value, liabs_value)
        }
    }

    /// checks if the user is bankrupt
    pub fn is_bankrupt(&self, group: &CypherGroup) -> bool {
        let quote_position = self.get_position(QUOTE_TOKEN_IDX).unwrap();
        let mut largest_deposit_value =
            quote_position.total_deposits(group.get_cypher_token(QUOTE_TOKEN_IDX).unwrap());
        let mut lowest_borrow_price = if quote_position.deriv_base_borrows() > I80F48::ZERO {
            I80F48::ONE
        } else {
            I80F48::MAX
        };
        for position in self.iter_positions() {
            let market_idx = position.market_idx as usize;
            let market_price = I80F48::from(
                group
                    .get_cypher_market(market_idx)
                    .unwrap()
                    .derivative_info
                    .market_price,
            );
            // we can use native deposits here because cAssets don't accrue interest
            let deposit_value = position.deriv_base_deposits() * market_price;
            largest_deposit_value = I80F48::max(largest_deposit_value, deposit_value);
            if position.deriv_base_borrows() > I80F48::ZERO {
                lowest_borrow_price = lowest_borrow_price.min(market_price);
            }
        }

        if lowest_borrow_price == u64::MAX {
            return false;
        }

        let liq_fee = group.liq_liqor_fee() + group.liq_insurance_fee();
        let collateral_for_min_borrow_unit = liq_fee * lowest_borrow_price;

        collateral_for_min_borrow_unit > largest_deposit_value
    }
}

impl UserPosition {
    /// gets the base deposit amount
    pub fn spot_base_deposits(&self) -> I80F48 {
        I80F48::from_bits(self.spot_base_deposits)
    }

    /// gets the base borrows amount
    pub fn spot_base_borrows(&self) -> I80F48 {
        I80F48::from_bits(self.spot_base_borrows)
    }

    /// gets the base deposit amount
    pub fn deriv_base_deposits(&self) -> I80F48 {
        I80F48::from_bits(self.spot_base_deposits)
    }

    /// gets the base borrows amount
    pub fn deriv_base_borrows(&self) -> I80F48 {
        I80F48::from_bits(self.spot_base_borrows)
    }

    /// gets the user's total deposits adjusted for the token's deposit index
    pub fn total_deposits(&self, cypher_token: &CypherToken) -> I80F48 {
        self.spot_base_deposits() * cypher_token.deposit_index()
    }

    /// gets the user's total borrows adjusted for the token's borrow index
    pub fn total_borrows(&self, cypher_token: &CypherToken) -> I80F48 {
        self.spot_base_borrows() * cypher_token.borrow_index()
    }
}
