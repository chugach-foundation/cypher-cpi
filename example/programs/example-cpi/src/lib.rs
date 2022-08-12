use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Token, TokenAccount},
    dex::Dex
};
use std::mem::size_of;
use cypher::{
    cpi::{
        accounts::{
            CreateCypherUser, DepositCollateral, WithdrawCollateral,
            NoOpNewOrderV3 as NewOrderV3, NoOpCancelOrder as CancelOrderV2,
            NoOpNewOrderV3Dex as NewOrderV3Dex, NoOpCancelOrderDex as CancelOrderV2Dex
        },
        create_cypher_user, deposit_collateral, withdraw_collateral
    },
    serum_cpi::{
        new_order_v3, cancel_order_v2, settle_funds
    },
    program::Cypher,
    CypherGroup, CypherUser, quote_mint
};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod example_cpi {
    use super::*;
    use serum_dex::instruction::MarketInstruction;

    pub fn entry(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> Result<()> {
        let mut accounts = accounts;
        let mut bumps = std::collections::BTreeMap::new();
        let ix = MarketInstruction::unpack(data);

        match ix {
            Some(MarketInstruction::NewOrderV3(ix)) => {
                let mut accounts =
                    Accounts::try_accounts(program_id, &mut accounts, &[], &mut bumps)?;
                let ctx = Context::new(program_id, &mut accounts, &[], bumps);
                new_order::handler(ctx, ix)
            }
            Some(MarketInstruction::CancelOrderV2(ix)) => {
                let mut accounts =
                    Accounts::try_accounts(program_id, &mut accounts, &[], &mut bumps)?;
                let ctx = Context::new(program_id, &mut accounts, &[], bumps);
                cancel_order::handler(ctx, ix)
            }
            _ => unimplemented!()
        }
    }

    pub fn initialize_user(
        ctx: Context<InitializeUser>,
        wrapper_bump: u8,
        cypher_user_bump: u8,
        account_number: u64,
    ) -> Result<()> {
        let wrapper = &mut ctx.accounts.wrapper;
        let cypher_group = &ctx.accounts.cypher_group;
        let cypher_user = &ctx.accounts.cypher_user;
        let admin = &ctx.accounts.admin;

        let cpi_accounts = CreateCypherUser {
            cypher_group: cypher_group.to_account_info(),
            cypher_user: cypher_user.to_account_info(),
            owner: wrapper.to_account_info(),
            payer: admin.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info()
        };

        let cypher_group_key = cypher_group.key();
        let admin_key = admin.key();
        let signer_seeds = &[
            b"account_wrapper",
            cypher_group_key.as_ref(),
            admin_key.as_ref(),
            &[wrapper_bump]
        ];
        let signer_seeds = [&signer_seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.cypher_program.to_account_info(),
            cpi_accounts,
            &signer_seeds
        );

        create_cypher_user(cpi_ctx, cypher_user_bump, account_number)?;
        wrapper.bump = [wrapper_bump];
        wrapper.admin = admin.key();
        Ok(())
    }

    pub fn deposit(
        ctx: Context<Deposit>,
        amount: u64,
    ) -> Result<()> {
        let wrapper = &ctx.accounts.wrapper;
        let cypher_group = &ctx.accounts.cypher_group;
        let cypher_user = &ctx.accounts.cypher_user;
        let cypher_pc_vault = &ctx.accounts.cypher_pc_vault;
        let source_token_account = &ctx.accounts.source_token_account;
        let token_program = &ctx.accounts.token_program;
        let admin = &ctx.accounts.admin;

        let cpi_accounts = DepositCollateral {
            cypher_group: cypher_group.to_account_info(),
            cypher_user: cypher_user.to_account_info(),
            user_signer: wrapper.to_account_info(),
            cypher_pc_vault: cypher_pc_vault.to_account_info(),
            deposit_from: source_token_account.to_account_info(),
            token_program: token_program.to_account_info(),
        };

        let cypher_group_key = cypher_group.key();
        let admin_key = admin.key();
        let signer_seeds = &[
            b"account_wrapper",
            cypher_group_key.as_ref(),
            admin_key.as_ref(),
            &wrapper.bump
        ];
        let signer_seeds = [&signer_seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.cypher_program.to_account_info(),
            cpi_accounts,
            &signer_seeds
        );

        deposit_collateral(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn withdraw(
        ctx: Context<Withdraw>,
        amount: u64,
    ) -> Result<()> {
        let wrapper = &ctx.accounts.wrapper;
        let cypher_group = &ctx.accounts.cypher_group;
        let cypher_user = &ctx.accounts.cypher_user;
        let cypher_pc_vault = &ctx.accounts.cypher_pc_vault;
        let destination_token_account = &ctx.accounts.destination_token_account;
        let token_program = &ctx.accounts.token_program;
        let vault_signer = &ctx.accounts.vault_signer;
        let admin = &ctx.accounts.admin;

        let cpi_accounts = WithdrawCollateral {
            cypher_group: cypher_group.to_account_info(),
            cypher_user: cypher_user.to_account_info(),
            user_signer: wrapper.to_account_info(),
            cypher_pc_vault: cypher_pc_vault.to_account_info(),
            withdraw_to: destination_token_account.to_account_info(),
            token_program: token_program.to_account_info(),
            vault_signer: vault_signer.to_account_info()
        };

        let cypher_group_key = cypher_group.key();
        let admin_key = admin.key();
        let signer_seeds = &[
            b"account_wrapper",
            cypher_group_key.as_ref(),
            admin_key.as_ref(),
            &wrapper.bump
        ];
        let signer_seeds = [&signer_seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.cypher_program.to_account_info(),
            cpi_accounts,
            &signer_seeds
        );

        withdraw_collateral(cpi_ctx, amount)?;

        Ok(())
    }
}

mod new_order {
    use super::*;
    use serum_dex::instruction::NewOrderInstructionV3;

    pub fn handler(
        ctx: Context<NewOrder>,
        mut ix_data: NewOrderInstructionV3,
    ) -> Result<()> {
        let wrapper = &ctx.accounts.wrapper;
        let cypher_group = &ctx.accounts.cypher_group;
        let cypher_user = &ctx.accounts.cypher_user;
        let cypher_pc_vault = &ctx.accounts.cypher_pc_vault;
        let cypher_c_asset_vault = &ctx.accounts.cypher_c_asset_vault;
        let price_history = &ctx.accounts.price_history;
        let c_asset_mint = &ctx.accounts.c_asset_mint;
        let cypher_vault_signer = &ctx.accounts.cypher_vault_signer;
        let admin = &ctx.accounts.admin;

        let cpi_accounts = NewOrderV3 {
            cypher_group: cypher_group.to_account_info(),
            cypher_user: cypher_user.to_account_info(),
            user_signer: wrapper.to_account_info(),
            cypher_pc_vault: cypher_pc_vault.to_account_info(),
            cypher_c_asset_vault: cypher_c_asset_vault.to_account_info(),
            c_asset_mint: c_asset_mint.to_account_info(),
            price_history: price_history.to_account_info(),
            vault_signer: cypher_vault_signer.to_account_info(),
            NoOpNewOrderV3dex: NewOrderV3Dex {
                market: ctx.accounts.cypher_market.to_account_info(),
                open_orders: ctx.accounts.open_orders.to_account_info(),
                event_q: ctx.accounts.event_queue.to_account_info(),
                req_q: ctx.accounts.request_queue.to_account_info(),
                asks: ctx.accounts.asks.to_account_info(),
                bids: ctx.accounts.bids.to_account_info(),
                coin_vault: ctx.accounts.coin_vault.to_account_info(),
                pc_vault: ctx.accounts.pc_vault.to_account_info(),
                vault_signer: ctx.accounts.dex_vault_signer.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                dex_program: ctx.accounts.dex_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            }
        };

        let cypher_group_key = cypher_group.key();
        let admin_key = admin.key();
        let signer_seeds = &[
            b"account_wrapper",
            cypher_group_key.as_ref(),
            admin_key.as_ref(),
            &wrapper.bump
        ];
        let signer_seeds = [&signer_seeds[..]];

        let cypher_program = &ctx.accounts.cypher_program;

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.cypher_program.to_account_info(),
            cpi_accounts,
            &signer_seeds
        );

        new_order_v3(
            cpi_ctx,
            ix_data
        )?;

        Ok(())
    }
}

mod cancel_order {
    use super::*;
    use serum_dex::instruction::CancelOrderInstructionV2;

    pub fn handler(
        ctx: Context<CancelOrder>,
        mut ix_data: CancelOrderInstructionV2,
    ) -> Result<()> {
        let wrapper = &ctx.accounts.wrapper;
        let cypher_group = &ctx.accounts.cypher_group;
        let cypher_user = &ctx.accounts.cypher_user;
        let cypher_c_asset_vault = &ctx.accounts.cypher_c_asset_vault;
        let cypher_pc_vault = &ctx.accounts.cypher_pc_vault;
        let c_asset_mint = &ctx.accounts.c_asset_mint;
        let cypher_vault_signer = &ctx.accounts.cypher_vault_signer;
        let admin = &ctx.accounts.admin;

        let cpi_accounts =  CancelOrderV2 {
            cypher_group: cypher_group.to_account_info(),
            cypher_user: cypher_user.to_account_info(),
            user_signer: wrapper.to_account_info(),
            cypher_c_asset_vault: cypher_c_asset_vault.to_account_info(),
            cypher_pc_vault: cypher_pc_vault.to_account_info(),
            c_asset_mint: c_asset_mint.to_account_info(),
            vault_signer: cypher_vault_signer.to_account_info(),
            NoOpCancelOrderdex: CancelOrderV2Dex {
                market: ctx.accounts.cypher_market.to_account_info(),
                open_orders: ctx.accounts.open_orders.to_account_info(),
                prune_authority: ctx.accounts.prune_authority.to_account_info(),
                event_q: ctx.accounts.event_queue.to_account_info(),
                asks: ctx.accounts.asks.to_account_info(),
                bids: ctx.accounts.bids.to_account_info(),
                coin_vault: ctx.accounts.coin_vault.to_account_info(),
                pc_vault: ctx.accounts.pc_vault.to_account_info(),
                vault_signer: ctx.accounts.dex_vault_signer.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                dex_program: ctx.accounts.dex_program.to_account_info(),
            }
        };

        let cypher_group_key = cypher_group.key();
        let admin_key = admin.key();
        let signer_seeds = &[
            b"account_wrapper",
            cypher_group_key.as_ref(),
            admin_key.as_ref(),
            &wrapper.bump
        ];
        let signer_seeds = &[&signer_seeds[..]];

        let cypher_program = &ctx.accounts.cypher_program;

        let cpi_ctx = CpiContext::new_with_signer(
            cypher_program.to_account_info(),
            cpi_accounts,
            signer_seeds
        );

        cancel_order_v2(
            cpi_ctx,
            ix_data
        )?;

        Ok(())
    }
}

#[account]
pub struct UserWrapper {
    pub bump: [u8; 1],

    pub admin: Pubkey,
}

#[derive(Accounts)]
#[instruction(
    wrapper_bump: u8
)]
pub struct NewOrder<'info> {
    #[account(
        seeds = [
            b"account_wrapper",
            cypher_group.key().as_ref(),
            admin.key().as_ref(),
        ],
        bump = wrapper_bump,
        has_one = admin
    )]
    pub wrapper: Box<Account<'info, UserWrapper>>,
    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub cypher_group: AccountLoader<'info, CypherGroup>,
    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub cypher_user: AccountInfo<'info>,
    /// CHECK: Address validation done.
    #[account(
        mut,
        address = cypher_group.load()?.quote_vault()
    )]
    pub cypher_pc_vault: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub cypher_c_asset_vault: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    pub cypher_vault_signer: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    pub price_history: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    pub c_asset_mint: AccountInfo<'info>,
    /// Serum DEX accounts
    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub cypher_market: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub request_queue: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub bids: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub asks: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub coin_vault: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub pc_vault: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    pub dex_vault_signer: AccountInfo<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub cypher_program: Program<'info, Cypher>,
    pub token_program: Program<'info, Token>,
    pub dex_program: Program<'info, Dex>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(
    wrapper_bump: u8
)]
pub struct CancelOrder<'info> {
    #[account(
        seeds = [
            b"account_wrapper",
            cypher_group.key().as_ref(),
            admin.key().as_ref(),
        ],
        bump = wrapper_bump,
        has_one = admin
    )]
    pub wrapper: Box<Account<'info, UserWrapper>>,
    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub cypher_group: AccountLoader<'info, CypherGroup>,
    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub cypher_user: AccountInfo<'info>,
    /// CHECK: Address validation done.
    #[account(
        mut,
        address = cypher_group.load()?.quote_vault()
    )]
    pub cypher_pc_vault: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub cypher_c_asset_vault: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    pub c_asset_mint: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    pub cypher_vault_signer: AccountInfo<'info>,
    /// Serum DEX accounts
    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub cypher_market: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    pub prune_authority: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub bids: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub asks: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub coin_vault: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub pc_vault: AccountInfo<'info>,
    /// CHECK: Checked through CPI call.
    pub dex_vault_signer: AccountInfo<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub cypher_program: Program<'info, Cypher>,
    pub token_program: Program<'info, Token>,
    pub dex_program: Program<'info, Dex>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(
    wrapper_bump: u8,
)]
pub struct Deposit<'info> {
    #[account(
        seeds = [
            b"account_wrapper",
            cypher_group.key().as_ref(),
            admin.key().as_ref(),
        ],
        bump = wrapper_bump,
        has_one = admin
    )]
    pub wrapper: Box<Account<'info, UserWrapper>>,

    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub cypher_group: AccountLoader<'info, CypherGroup>,

    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub cypher_user: AccountInfo<'info>,

    /// CHECK: Address validation done.
    #[account(
        mut,
        address = cypher_group.load()?.quote_vault()
    )]
    pub cypher_pc_vault: AccountInfo<'info>,

    #[account(
        mut,
        constraint = source_token_account.mint == quote_mint::ID,
        constraint = source_token_account.owner == admin.key() || source_token_account.delegate.unwrap() == admin.key()
    )]
    pub source_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub admin: Signer<'info>,

    pub cypher_program: Program<'info, Cypher>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(
    wrapper_bump: u8,
)]
pub struct Withdraw<'info> {
    #[account(
        seeds = [
            b"account_wrapper",
            cypher_group.key().as_ref(),
            admin.key().as_ref(),
        ],
        bump = wrapper_bump,
        has_one = admin
    )]
    pub wrapper: Box<Account<'info, UserWrapper>>,

    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub cypher_group: AccountLoader<'info, CypherGroup>,

    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub cypher_user: AccountLoader<'info, CypherUser>,

    /// CHECK: Address validation done.
    #[account(
        mut,
        address = cypher_group.load()?.quote_vault()
    )]
    pub cypher_pc_vault: AccountInfo<'info>,

    /// CHECK: Address validation done.
    #[account(
        mut,
        address = cypher_group.load()?.vault_signer
    )]
    pub vault_signer: AccountInfo<'info>,

    #[account(
        mut,
        constraint = destination_token_account.mint == quote_mint::ID,
        constraint = destination_token_account.owner == admin.key() || destination_token_account.delegate.unwrap() == admin.key()
    )]
    pub destination_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub admin: Signer<'info>,

    pub cypher_program: Program<'info, Cypher>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(
    wrapper_bump: u8,
    cypher_user_bump: u8,
)]
pub struct InitializeUser<'info> {
    #[account(
        init,
        seeds = [
            b"account_wrapper",
            cypher_group.key().as_ref(),
            admin.key().as_ref(),
        ],
        bump,
        payer = admin,
        space = 8 + size_of::<UserWrapper>()
    )]
    pub wrapper: Box<Account<'info, UserWrapper>>,

    /// CHECK: Checked through CPI call.
    pub cypher_group: AccountLoader<'info, CypherGroup>,

    /// CHECK: Checked through CPI call.
    #[account(mut)]
    pub cypher_user: AccountLoader<'info, CypherUser>,
    
    #[account(mut)]
    pub admin: Signer<'info>,

    pub cypher_program: Program<'info, Cypher>,
    pub system_program: Program<'info, System>,
}