#![allow(dead_code)]
use {
    crate::accounts::{
        DepositCollateral, InitCypherUser, LiquidateCollateral, NoOpCancelOrder as CancelOrder,
        NoOpCancelOrderDex as CancelOrderDex, NoOpCloseOpenOrders as CloseOpenOrders,
        NoOpInitOpenOrders as InitOpenOrders, NoOpNewOrderV3 as NewOrderV3,
        NoOpNewOrderV3Dex as NewOrderV3Dex, NoOpSettleFunds as SettleFunds,
        NoOpSettleFundsDex as SettleFundsDex, SettlePosition, WithdrawCollateral,
    },
    crate::constants::*,
    arrayref::array_ref,
    anchor_lang::{prelude::*, system_program, ZeroCopy},
    anchor_spl::{dex, token, token::spl_token},
    bytemuck::{bytes_of, from_bytes, Pod},
    serum_dex::instruction::{CancelOrderInstructionV2, MarketInstruction, NewOrderInstructionV3},
    sha2::{Digest, Sha256},
    solana_sdk::{instruction::Instruction, sysvar::SysvarId, account::Account},
};

pub trait ToPubkey {
    fn to_pubkey(&self) -> Pubkey;
}

impl ToPubkey for [u64; 4] {
    fn to_pubkey(&self) -> Pubkey {
        Pubkey::new(bytes_of(self))
    }
}

pub fn get_zero_copy_account<T: ZeroCopy + Owner>(solana_account: &Account) -> Box<T> {
    let data = &solana_account.data.as_slice();
    let disc_bytes = array_ref![data, 0, 8];
    assert_eq!(disc_bytes, &T::discriminator());
    Box::new(*from_bytes::<T>(&data[8..std::mem::size_of::<T>() + 8]))
}

pub fn parse_dex_account<T: Pod>(data: Vec<u8>) -> T {
    let data_len = data.len() - 12;
    let (_, rest) = data.split_at(5);
    let (mid, _) = rest.split_at(data_len);
    *from_bytes(mid)
}

pub fn gen_dex_vault_signer_key(nonce: u64, dex_market_pk: &Pubkey) -> Pubkey {
    let seeds = [dex_market_pk.as_ref(), bytes_of(&nonce)];
    Pubkey::create_program_address(&seeds, &dex::id()).unwrap()
}

pub fn derive_dex_market_authority(dex_market_pk: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[B_DEX_MARKET_AUTHORITY, dex_market_pk.as_ref()],
        &crate::id(),
    )
    .0
}

pub fn derive_cypher_user_address(cypher_group_pk: &Pubkey, owner_pk: &Pubkey) -> (Pubkey, u8) {
    let (address, bump) = Pubkey::find_program_address(
        &[
            B_CYPHER_USER,
            cypher_group_pk.as_ref(),
            &owner_pk.to_bytes(),
        ],
        &crate::id(),
    );

    (address, bump)
}

pub fn derive_open_orders_address(dex_market_pk: &Pubkey, cypher_user_pk: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            B_OPEN_ORDERS,
            dex_market_pk.as_ref(),
            cypher_user_pk.as_ref(),
        ],
        &crate::id(),
    )
}

#[derive(Clone, Default)]
pub struct Hasher {
    hasher: Sha256,
}

impl Hasher {
    pub fn hash(&mut self, val: &[u8]) {
        self.hasher.update(val);
    }
    pub fn hashv(&mut self, vals: &[&[u8]]) {
        for val in vals {
            self.hash(val);
        }
    }
    pub fn result(self) -> [u8; 32] {
        <[u8; 32]>::try_from(self.hasher.finalize().as_slice()).unwrap()
    }
}

fn hashv(vals: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Hasher::default();
    hasher.hashv(vals);
    hasher.result()
}

fn hash(val: &[u8]) -> [u8; 32] {
    hashv(&[val])
}

fn sighash(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", namespace, name);

    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(&hash(preimage.as_bytes())[..8]);
    sighash
}

fn get_ix_data(name: &str, mut ix_data: Vec<u8>) -> Vec<u8> {
    let mut data = sighash("global", name).to_vec();
    data.append(&mut ix_data);
    data
}

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
            "liquuidate_collateral",
            AnchorSerialize::try_to_vec(&ix_data).unwrap(),
        ),
        program_id: crate::id(),
    }
}

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
