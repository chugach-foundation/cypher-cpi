#![allow(dead_code)]
use {
    crate::constants::*,
    anchor_lang::prelude::*,
    anchor_spl::dex,
    bytemuck::{bytes_of, from_bytes, Pod},
};

#[cfg(feature = "client")]
use {
    anchor_lang::ZeroCopy,
    arrayref::array_ref,
    solana_sdk::account::Account,
};

#[cfg(feature = "client")]
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

pub fn derive_cypher_user_address_with_number(cypher_group_pk: &Pubkey, owner_pk: &Pubkey, account_number: u64) -> (Pubkey, u8) {
    let (address, bump) = Pubkey::find_program_address(
        &[
            B_CYPHER_USER,
            cypher_group_pk.as_ref(),
            &owner_pk.to_bytes(),
            &account_number.to_le_bytes()
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
