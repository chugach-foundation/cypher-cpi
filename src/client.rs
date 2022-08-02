#![allow(dead_code)]
use {
    crate::accounts::{
        DepositCollateral as DepositCollateralAccounts, InitCypherUser as InitCypherUserAccounts,
        NoOpCancelOrders as CancelOrdersAccounts, NoOpNewOrderV3 as NewOrderV3Accounts,
        SettlePosition as SettlePositionAccounts, WithdrawCollateral as WithdrawCollateralAccounts,
        LiquidateCollateral as LiquidateCollateralAccounts,
    },
    crate::instruction::{
        DepositCollateral as DepositCollateralInstruction,
        InitCypherUser as InitCypherUserInstruction, NoOpCancelOrders as CancelOrdersInstruction,
        NoOpNewOrderV3 as NewOrderV3Instruction, SettlePosition as SettlePositionInstruction,
        WithdrawCollateral as WithdrawCollateralInstruction, LiquidateCollateral as LiquidateCollateralInstruction
    },
    anchor_lang::{prelude::Pubkey, system_program, AnchorSerialize, ToAccountMetas},
    anchor_spl::token,
    solana_sdk::instruction::Instruction,
};

pub fn init_cypher_user(
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

pub fn deposit_collateral(
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

pub fn withdraw_collateral(
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

pub fn liquidate_collateral(
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
        liqee_cypher_user: *liqee_cypher_user
    };
    let ix_data = LiquidateCollateralInstruction {
        _asset_mint: *asset_mint,
        _liab_mint: *liability_mint
    };

    Instruction {
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.try_to_vec().unwrap(),
        program_id: crate::id(),        
    }
}

pub fn settle_position(
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