use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::spl_token;
use sha2::{Digest, Sha256};
use solana_sdk::instruction::Instruction;

anchor_gen::generate_cpi_interface!(idl_path = "idl.json");

#[cfg(feature = "devnet")]
declare_id!("7njrvFJx4NJQvzywv1LdnPwzYYTSh1wWgGL5vkwTUuSS");
#[cfg(not(feature = "devnet"))]
declare_id!("7njrvFJx4NJQvzywv1LdnPwzYYTSh1wWgGL5vkwTUuSS");

pub mod quote_mint {
    use anchor_lang::declare_id;
    #[cfg(feature = "mainnet-beta")]
    declare_id!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    #[cfg(not(feature = "mainnet-beta"))]
    declare_id!("DPhNUKVhnrkdbq37GUgTUBRbZLsvziX1p5e5YUXyjBsb");
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

pub fn get_request_airdrop_ix(token_account: &Pubkey, amount: u64) -> Instruction {
    let accounts = crate::accounts::FaucetToUser {
        faucet_info: Pubkey::from_str("9euKg1WZtat7iupnqZJPhVFUq1Eg3VJVAdAsv5T88Nf1").unwrap(),
        mint: quote_mint::ID,
        mint_authority: Pubkey::from_str("ALtS7g1kR3T1YkAZFo8SwKP36nhCKVf11Eh4xDsxKY1U").unwrap(),
        target: *token_account,
        token_program: spl_token::ID,
    };
    let ix_data = crate::instruction::FaucetToUser { _amount: amount };
    Instruction {
        accounts: accounts.to_account_metas(Some(false)),
        data: get_ix_data(
            "faucet_to_user",
            AnchorSerialize::try_to_vec(&ix_data).unwrap(),
        ),
        program_id: crate::id(),
    }
}
