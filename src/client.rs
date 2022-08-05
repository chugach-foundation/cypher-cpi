#![allow(dead_code)]
use {
    crate::accounts::{
        DepositCollateral as DepositCollateralAccounts, InitCypherUser as InitCypherUserAccounts,
        LiquidateCollateral as LiquidateCollateralAccounts, NoOpCancelOrder as CancelOrderAccounts,
        NoOpCancelOrderDex as CancelOrderDexAccounts, NoOpNewOrderV3 as NewOrderV3Accounts,
        NoOpNewOrderV3Dex as NewOrderV3DexAccounts, NoOpSettleFunds as SettleFundsAccounts,
        NoOpSettleFundsDex as SettleFundsDexAccounts, SettlePosition as SettlePositionAccounts,
        WithdrawCollateral as WithdrawCollateralAccounts,
    },
    crate::instruction::{
        DepositCollateral as DepositCollateralInstruction,
        InitCypherUser as InitCypherUserInstruction,
        LiquidateCollateral as LiquidateCollateralInstruction,
        SettlePosition as SettlePositionInstruction,
        WithdrawCollateral as WithdrawCollateralInstruction,
    },
    anchor_lang::{prelude::*, system_program},
    anchor_spl::{dex, token, token::spl_token},
    bytemuck::bytes_of,
    serum_dex::instruction::{CancelOrderInstructionV2, MarketInstruction, NewOrderInstructionV3},
    solana_sdk::{instruction::Instruction, sysvar::SysvarId},
};

pub fn gen_dex_vault_signer_key(nonce: u64, dex_market_pk: &Pubkey) -> Pubkey {
    let seeds = [dex_market_pk.as_ref(), bytes_of(&nonce)];
    Pubkey::create_program_address(&seeds, &dex::id()).unwrap()
}

pub fn derive_dex_market_authority(dex_market_pk: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"dex_market_authority", dex_market_pk.as_ref()],
        &crate::id(),
    )
}

pub fn init_cypher_user_ix(
    cypher_group: &Pubkey,
    cypher_user: &Pubkey,
    owner: &Pubkey,
    bump: u8,
) -> Instruction {
    let accounts = InitCypherUserAccounts {
        cypher_group: *cypher_group,
        cypher_user: *cypher_user,
        owner: *owner,
        system_program: system_program::ID,
    };
    let ix_data = InitCypherUserInstruction { _bump: bump };

    Instruction {
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.try_to_vec().unwrap(),
        program_id: crate::id(),
    }
}

pub fn deposit_collateral_ix(
    cypher_group: &Pubkey,
    cypher_user: &Pubkey,
    cypher_pc_vault: &Pubkey,
    owner: &Pubkey,
    source_token_account: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = DepositCollateralAccounts {
        cypher_group: *cypher_group,
        cypher_user: *cypher_user,
        user_signer: *owner,
        cypher_pc_vault: *cypher_pc_vault,
        deposit_from: *source_token_account,
        token_program: token::ID,
    };
    let ix_data = DepositCollateralInstruction { _amount: amount };

    Instruction {
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.try_to_vec().unwrap(),
        program_id: crate::id(),
    }
}

pub fn withdraw_collateral_ix(
    cypher_group: &Pubkey,
    cypher_user: &Pubkey,
    cypher_pc_vault: &Pubkey,
    vault_signer: &Pubkey,
    owner: &Pubkey,
    destination_token_account: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = WithdrawCollateralAccounts {
        cypher_group: *cypher_group,
        cypher_user: *cypher_user,
        user_signer: *owner,
        vault_signer: *vault_signer,
        cypher_pc_vault: *cypher_pc_vault,
        withdraw_to: *destination_token_account,
        token_program: token::ID,
    };
    let ix_data = WithdrawCollateralInstruction { _amount: amount };

    Instruction {
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.try_to_vec().unwrap(),
        program_id: crate::id(),
    }
}

pub fn liquidate_collateral_ix(
    cypher_group: &Pubkey,
    cypher_user: &Pubkey,
    owner: &Pubkey,
    liqee_cypher_user: &Pubkey,
    asset_mint: &Pubkey,
    liability_mint: &Pubkey,
) -> Instruction {
    let accounts = LiquidateCollateralAccounts {
        cypher_group: *cypher_group,
        cypher_user: *cypher_user,
        user_signer: *owner,
        liqee_cypher_user: *liqee_cypher_user,
    };
    let ix_data = LiquidateCollateralInstruction {
        _asset_mint: *asset_mint,
        _liab_mint: *liability_mint,
    };

    Instruction {
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.try_to_vec().unwrap(),
        program_id: crate::id(),
    }
}

pub fn settle_position_ix(
    cypher_group: &Pubkey,
    cypher_user: &Pubkey,
    c_asset_mint: &Pubkey,
) -> Instruction {
    let accounts = SettlePositionAccounts {
        cypher_group: *cypher_group,
        cypher_user: *cypher_user,
        c_asset_mint: *c_asset_mint,
    };
    let ix_data = SettlePositionInstruction {};

    Instruction {
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.try_to_vec().unwrap(),
        program_id: crate::id(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn new_order_v3_ix(
    cypher_group: &Pubkey,
    cypher_market: &Pubkey,
    vault_signer: &Pubkey,
    price_history: &Pubkey,
    cypher_user: &Pubkey,
    user_signer: &Pubkey,
    c_asset_mint: &Pubkey,
    cypher_c_asset_vault: &Pubkey,
    cypher_pc_vault: &Pubkey,
    dex_market: &Pubkey,
    open_orders: &Pubkey,
    request_queue: &Pubkey,
    event_queue: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    coin_vault: &Pubkey,
    pc_vault: &Pubkey,
    dex_vault_signer: &Pubkey,
    data: NewOrderInstructionV3,
) -> Instruction {
    let accounts = NewOrderV3Accounts {
        cypher_group: *cypher_group,
        vault_signer: *vault_signer,
        price_history: *price_history,
        cypher_user: *cypher_user,
        user_signer: *user_signer,
        c_asset_mint: *c_asset_mint,
        cypher_c_asset_vault: *cypher_c_asset_vault,
        cypher_pc_vault: *cypher_pc_vault,
        NoOpNewOrderV3dex: NewOrderV3DexAccounts {
            market: *dex_market,
            open_orders: *open_orders,
            req_q: *request_queue,
            event_q: *event_queue,
            bids: *bids,
            asks: *asks,
            coin_vault: *coin_vault,
            pc_vault: *pc_vault,
            vault_signer: *dex_vault_signer,
            rent: Rent::id(),
            token_program: spl_token::id(),
            dex_program: dex::id(),
        },
    };

    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: MarketInstruction::NewOrderV3(data).pack(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn cancel_order_ix(
    cypher_group: &Pubkey,
    cypher_market: &Pubkey,
    vault_signer: &Pubkey,
    price_history: &Pubkey,
    cypher_user: &Pubkey,
    user_signer: &Pubkey,
    c_asset_mint: &Pubkey,
    cypher_c_asset_vault: &Pubkey,
    cypher_pc_vault: &Pubkey,
    dex_market: &Pubkey,
    prune_authority: &Pubkey,
    open_orders: &Pubkey,
    event_queue: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    coin_vault: &Pubkey,
    pc_vault: &Pubkey,
    dex_vault_signer: &Pubkey,
    data: CancelOrderInstructionV2,
) -> Instruction {
    let accounts = CancelOrderAccounts {
        cypher_group: *cypher_group,
        vault_signer: *vault_signer,
        cypher_user: *cypher_user,
        user_signer: *user_signer,
        c_asset_mint: *c_asset_mint,
        cypher_c_asset_vault: *cypher_c_asset_vault,
        cypher_pc_vault: *cypher_pc_vault,
        NoOpCancelOrderdex: CancelOrderDexAccounts {
            market: *dex_market,
            prune_authority: *prune_authority,
            open_orders: *open_orders,
            event_q: *event_queue,
            bids: *bids,
            asks: *asks,
            coin_vault: *coin_vault,
            pc_vault: *pc_vault,
            vault_signer: *dex_vault_signer,
            token_program: spl_token::id(),
            dex_program: dex::id(),
        },
    };

    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: MarketInstruction::CancelOrderV2(data).pack(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn cancel_order_by_client_id_ix(
    cypher_group: &Pubkey,
    cypher_market: &Pubkey,
    vault_signer: &Pubkey,
    price_history: &Pubkey,
    cypher_user: &Pubkey,
    user_signer: &Pubkey,
    c_asset_mint: &Pubkey,
    cypher_c_asset_vault: &Pubkey,
    cypher_pc_vault: &Pubkey,
    dex_market: &Pubkey,
    prune_authority: &Pubkey,
    open_orders: &Pubkey,
    event_queue: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    coin_vault: &Pubkey,
    pc_vault: &Pubkey,
    dex_vault_signer: &Pubkey,
    client_id: u64,
) -> Instruction {
    let accounts = CancelOrderAccounts {
        cypher_group: *cypher_group,
        vault_signer: *vault_signer,
        cypher_user: *cypher_user,
        user_signer: *user_signer,
        c_asset_mint: *c_asset_mint,
        cypher_c_asset_vault: *cypher_c_asset_vault,
        cypher_pc_vault: *cypher_pc_vault,
        NoOpCancelOrderdex: CancelOrderDexAccounts {
            market: *dex_market,
            prune_authority: *prune_authority,
            open_orders: *open_orders,
            event_q: *event_queue,
            bids: *bids,
            asks: *asks,
            coin_vault: *coin_vault,
            pc_vault: *pc_vault,
            vault_signer: *dex_vault_signer,
            token_program: spl_token::id(),
            dex_program: dex::id(),
        },
    };

    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: MarketInstruction::CancelOrderByClientIdV2(client_id).pack(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn settle_funds_ix(
    cypher_group: &Pubkey,
    cypher_market: &Pubkey,
    vault_signer: &Pubkey,
    price_history: &Pubkey,
    cypher_user: &Pubkey,
    user_signer: &Pubkey,
    c_asset_mint: &Pubkey,
    cypher_c_asset_vault: &Pubkey,
    cypher_pc_vault: &Pubkey,
    dex_market: &Pubkey,
    open_orders: &Pubkey,
    coin_vault: &Pubkey,
    pc_vault: &Pubkey,
    dex_vault_signer: &Pubkey,
) -> Instruction {
    let accounts = SettleFundsAccounts {
        cypher_group: *cypher_group,
        vault_signer: *vault_signer,
        cypher_user: *cypher_user,
        user_signer: *user_signer,
        c_asset_mint: *c_asset_mint,
        cypher_c_asset_vault: *cypher_c_asset_vault,
        cypher_pc_vault: *cypher_pc_vault,
        NoOpSettleFundsdex: SettleFundsDexAccounts {
            market: *dex_market,
            open_orders: *open_orders,
            coin_vault: *coin_vault,
            pc_vault: *pc_vault,
            vault_signer: *dex_vault_signer,
            token_program: spl_token::id(),
            dex_program: dex::id(),
        },
    };

    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: MarketInstruction::SettleFunds.pack(),
    }
}

pub fn consume_events_ix(
    cypher_group: &Pubkey,
    cypher_users: &[Pubkey],
    open_orders: &[Pubkey],
    dex_market: &Pubkey,
    event_queue: &Pubkey,
    crank_authority: &Pubkey,
    limit: u16,
) -> Instruction {
    let mut accounts = vec![AccountMeta::new(*cypher_group, false)];
    let users = cypher_users.iter().map(|pk| AccountMeta::new(*pk, false));
    let open_orders = open_orders.iter().map(|pk| AccountMeta::new(*pk, false));
    let rem = vec![
        AccountMeta::new(*dex_market, false),
        AccountMeta::new(*event_queue, false),
        AccountMeta::new_readonly(*crank_authority, false),
        AccountMeta::new_readonly(dex::id(), false),
    ];
    accounts.extend(users);
    accounts.extend(open_orders);
    accounts.extend(rem);

    Instruction {
        program_id: crate::id(),
        accounts,
        data: MarketInstruction::ConsumeEventsPermissioned(limit).pack(),
    }
}
