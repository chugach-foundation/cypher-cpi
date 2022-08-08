use anchor_lang::prelude::*;

#[cfg(feature = "client")]
use {
    anchor_discriminator::get_ix_data,
    anchor_spl::token::spl_token,
    solana_sdk::instruction::Instruction,
    std::str::FromStr,
};

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

#[cfg(feature = "client")]
pub fn request_airdrop_ix(token_account: &Pubkey, amount: u64) -> Instruction {
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
