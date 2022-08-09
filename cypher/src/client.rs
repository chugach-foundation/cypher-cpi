#![allow(dead_code)]
#[cfg(feature = "client")]
use {
    crate::accounts::{
        DepositCollateral, InitCypherUser, LiquidateCollateral, NoOpCancelOrder as CancelOrder,
        NoOpCancelOrderDex as CancelOrderDex, NoOpCloseOpenOrders as CloseOpenOrders,
        NoOpInitOpenOrders as InitOpenOrders, NoOpNewOrderV3 as NewOrderV3,
        NoOpNewOrderV3Dex as NewOrderV3Dex, NoOpSettleFunds as SettleFunds,
        NoOpSettleFundsDex as SettleFundsDex, SettlePosition, WithdrawCollateral,
        CloseCypherUser, SetDelegate, CreateCypherUser,
    },
    anchor_discriminator::get_ix_data,
    anchor_lang::{prelude::*, system_program},
    anchor_spl::{dex, token, token::spl_token},
    bytemuck::bytes_of,
    serum_dex::instruction::{CancelOrderInstructionV2, MarketInstruction, NewOrderInstructionV3},
    solana_sdk::{instruction::Instruction, sysvar::SysvarId},
};

#[cfg(feature = "client")]
pub trait ToPubkey {
    fn to_pubkey(&self) -> Pubkey;
}

#[cfg(feature = "client")]
impl ToPubkey for [u64; 4] {
    fn to_pubkey(&self) -> Pubkey {
        Pubkey::new(bytes_of(self))
    }
}

#[cfg(feature = "client")]
pub fn init_cypher_user_ix(
    cypher_group: &Pubkey,
    cypher_user: &Pubkey,
    owner: &Pubkey,
    bump: u8,
) -> Instruction {
    let accounts = InitCypherUser {
        cypher_group: *cypher_group,
        cypher_user: *cypher_user,
        owner: *owner,
        system_program: system_program::ID,
    };
    let ix_data = crate::instruction::InitCypherUser { _bump: bump };
    Instruction {
        accounts: accounts.to_account_metas(Some(false)),
        data: get_ix_data(
            "init_cypher_user",
            AnchorSerialize::try_to_vec(&ix_data).unwrap(),
        ),
        program_id: crate::id(),
    }
}

#[cfg(feature = "client")]
pub fn create_cypher_user_ix(
    cypher_group: &Pubkey,
    cypher_user: &Pubkey,
    owner: &Pubkey,
    payer: &Pubkey,
    bump: u8,
    account_number: u64,
) -> Instruction {
    let accounts = CreateCypherUser {
        cypher_group: *cypher_group,
        cypher_user: *cypher_user,
        owner: *owner,
        payer: *payer,
        system_program: system_program::ID,
    };
    let ix_data = crate::instruction::CreateCypherUser {
        _bump: bump,
        _account_number: account_number,
    };
    Instruction {
        accounts: accounts.to_account_metas(Some(false)),
        data: get_ix_data(
            "create_cypher_user",
            AnchorSerialize::try_to_vec(&ix_data).unwrap(),
        ),
        program_id: crate::id(),
    }
}

#[cfg(feature = "client")]
pub fn close_cypher_user_ix(
    cypher_group: &Pubkey,
    cypher_user: &Pubkey,
    owner: &Pubkey,
) -> Instruction {
    let accounts = CloseCypherUser {
        cypher_group: *cypher_group,
        cypher_user: *cypher_user,
        user_signer: *owner,
    };
    let ix_data = crate::instruction::CloseCypherUser {};
    Instruction {
        accounts: accounts.to_account_metas(Some(false)),
        data: get_ix_data(
            "close_cypher_user",
            AnchorSerialize::try_to_vec(&ix_data).unwrap(),
        ),
        program_id: crate::id(),
    }
}

#[cfg(feature = "client")]
pub fn set_delegate_ix(
    cypher_group: &Pubkey,
    cypher_user: &Pubkey,
    owner: &Pubkey,
    delegate: &Pubkey,
) -> Instruction {
    let accounts = SetDelegate {
        cypher_group: *cypher_group,
        cypher_user: *cypher_user,
        user_signer: *owner,
        delegate: *delegate,
    };
    let ix_data = crate::instruction::SetDelegate {};
    Instruction {
        accounts: accounts.to_account_metas(Some(false)),
        data: get_ix_data(
            "set_delegate",
            AnchorSerialize::try_to_vec(&ix_data).unwrap(),
        ),
        program_id: crate::id(),
    }
}

#[cfg(feature = "client")]
pub fn deposit_collateral_ix(
    cypher_group: &Pubkey,
    cypher_user: &Pubkey,
    cypher_pc_vault: &Pubkey,
    owner: &Pubkey,
    source_token_account: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = DepositCollateral {
        cypher_group: *cypher_group,
        cypher_user: *cypher_user,
        user_signer: *owner,
        cypher_pc_vault: *cypher_pc_vault,
        deposit_from: *source_token_account,
        token_program: token::ID,
    };
    let ix_data = crate::instruction::DepositCollateral { _amount: amount };
    Instruction {
        accounts: accounts.to_account_metas(Some(false)),
        data: get_ix_data(
            "deposit_collateral",
            AnchorSerialize::try_to_vec(&ix_data).unwrap(),
        ),
        program_id: crate::id(),
    }
}

#[cfg(feature = "client")]
pub fn withdraw_collateral_ix(
    cypher_group: &Pubkey,
    cypher_user: &Pubkey,
    cypher_pc_vault: &Pubkey,
    vault_signer: &Pubkey,
    owner: &Pubkey,
    destination_token_account: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = WithdrawCollateral {
        cypher_group: *cypher_group,
        cypher_user: *cypher_user,
        user_signer: *owner,
        vault_signer: *vault_signer,
        cypher_pc_vault: *cypher_pc_vault,
        withdraw_to: *destination_token_account,
        token_program: token::ID,
    };
    let ix_data = crate::instruction::WithdrawCollateral { _amount: amount };

    Instruction {
        accounts: accounts.to_account_metas(Some(false)),
        data: get_ix_data(
            "withdraw_collateral",
            AnchorSerialize::try_to_vec(&ix_data).unwrap(),
        ),
        program_id: crate::id(),
    }
}

#[cfg(feature = "client")]
pub fn liquidate_collateral_ix(
    cypher_group: &Pubkey,
    cypher_user: &Pubkey,
    owner: &Pubkey,
    liqee_cypher_user: &Pubkey,
    asset_mint: &Pubkey,
    liability_mint: &Pubkey,
) -> Instruction {
    let accounts = LiquidateCollateral {
        cypher_group: *cypher_group,
        cypher_user: *cypher_user,
        user_signer: *owner,
        liqee_cypher_user: *liqee_cypher_user,
    };
    let ix_data = crate::instruction::LiquidateCollateral {
        _asset_mint: *asset_mint,
        _liab_mint: *liability_mint,
    };

    Instruction {
        accounts: accounts.to_account_metas(Some(false)),
        data: get_ix_data(
            "liquidate_collateral",
            AnchorSerialize::try_to_vec(&ix_data).unwrap(),
        ),
        program_id: crate::id(),
    }
}

#[cfg(feature = "client")]
pub fn settle_position_ix(
    cypher_group: &Pubkey,
    cypher_user: &Pubkey,
    c_asset_mint: &Pubkey,
) -> Instruction {
    let accounts = SettlePosition {
        cypher_group: *cypher_group,
        cypher_user: *cypher_user,
        c_asset_mint: *c_asset_mint,
    };
    let ix_data = crate::instruction::SettlePosition {};

    Instruction {
        accounts: accounts.to_account_metas(Some(false)),
        data: get_ix_data(
            "settle_position",
            AnchorSerialize::try_to_vec(&ix_data).unwrap(),
        ),
        program_id: crate::id(),
    }
}

#[cfg(feature = "client")]
pub fn init_open_orders_ix(
    cypher_group: &Pubkey,
    cypher_user: &Pubkey,
    user_signer: &Pubkey,
    dex_market: &Pubkey,
    open_orders: &Pubkey,
    market_authority: &Pubkey,
) -> Instruction {
    let accounts = InitOpenOrders {
        cypher_group: *cypher_group,
        cypher_user: *cypher_user,
        user_signer: *user_signer,
        dex_market: *dex_market,
        init_oo_authority: *market_authority,
        open_orders: *open_orders,
        rent: Rent::id(),
        system_program: system_program::ID,
        dex_program: dex::id(),
    };

    Instruction {
        accounts: accounts.to_account_metas(Some(false)),
        data: MarketInstruction::InitOpenOrders.pack(),
        program_id: crate::id(),
    }
}

#[cfg(feature = "client")]
pub fn close_open_orders_ix(
    cypher_group: &Pubkey,
    cypher_user: &Pubkey,
    user_signer: &Pubkey,
    dex_market: &Pubkey,
    open_orders: &Pubkey,
) -> Instruction {
    let accounts = CloseOpenOrders {
        cypher_group: *cypher_group,
        cypher_user: *cypher_user,
        user_signer: *user_signer,
        dex_market: *dex_market,
        open_orders: *open_orders,
        dex_program: dex::id(),
    };

    Instruction {
        accounts: accounts.to_account_metas(Some(false)),
        data: MarketInstruction::CloseOpenOrders.pack(),
        program_id: crate::id(),
    }
}

#[cfg(feature = "client")]
#[allow(clippy::too_many_arguments)]
pub fn new_order_v3_ix(
    cypher_group: &Pubkey,
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
    let accounts = NewOrderV3 {
        cypher_group: *cypher_group,
        vault_signer: *vault_signer,
        price_history: *price_history,
        cypher_user: *cypher_user,
        user_signer: *user_signer,
        c_asset_mint: *c_asset_mint,
        cypher_c_asset_vault: *cypher_c_asset_vault,
        cypher_pc_vault: *cypher_pc_vault,
        NoOpNewOrderV3dex: NewOrderV3Dex {
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

#[cfg(feature = "client")]
#[allow(clippy::too_many_arguments)]
pub fn cancel_order_ix(
    cypher_group: &Pubkey,
    vault_signer: &Pubkey,
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
    let accounts = CancelOrder {
        cypher_group: *cypher_group,
        vault_signer: *vault_signer,
        cypher_user: *cypher_user,
        user_signer: *user_signer,
        c_asset_mint: *c_asset_mint,
        cypher_c_asset_vault: *cypher_c_asset_vault,
        cypher_pc_vault: *cypher_pc_vault,
        NoOpCancelOrderdex: CancelOrderDex {
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

#[cfg(feature = "client")]
#[allow(clippy::too_many_arguments)]
pub fn cancel_order_by_client_id_ix(
    cypher_group: &Pubkey,
    vault_signer: &Pubkey,
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
    let accounts = CancelOrder {
        cypher_group: *cypher_group,
        vault_signer: *vault_signer,
        cypher_user: *cypher_user,
        user_signer: *user_signer,
        c_asset_mint: *c_asset_mint,
        cypher_c_asset_vault: *cypher_c_asset_vault,
        cypher_pc_vault: *cypher_pc_vault,
        NoOpCancelOrderdex: CancelOrderDex {
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

#[cfg(feature = "client")]
#[allow(clippy::too_many_arguments)]
pub fn settle_funds_ix(
    cypher_group: &Pubkey,
    vault_signer: &Pubkey,
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
    let accounts = SettleFunds {
        cypher_group: *cypher_group,
        vault_signer: *vault_signer,
        cypher_user: *cypher_user,
        user_signer: *user_signer,
        c_asset_mint: *c_asset_mint,
        cypher_c_asset_vault: *cypher_c_asset_vault,
        cypher_pc_vault: *cypher_pc_vault,
        NoOpSettleFundsdex: SettleFundsDex {
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

#[cfg(feature = "client")]
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
