#![allow(dead_code)]
use {
    crate::cpi::accounts::{
        NoOpCancelOrder as CancelOrderCpi, NoOpCloseOpenOrders as CloseOpenOrdersCpi,
        NoOpInitOpenOrders as InitOpenOrdersCpi, NoOpNewOrderV3 as NewOrderV3Cpi,
        NoOpSettleFunds as SettleFundsCpi,
    },
    crate::client::{
        init_open_orders_ix, close_open_orders_ix,
        new_order_v3_ix, cancel_order_v2_ix, cancel_order_by_client_id_v2_ix, settle_funds_ix,
    },
    anchor_lang::{prelude::*, solana_program::{program::invoke_signed}},
    serum_dex::instruction::{CancelOrderInstructionV2, NewOrderInstructionV3},
};

pub fn init_open_orders<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, InitOpenOrdersCpi<'info>>,
) -> Result<()> {
    let ix = init_open_orders_ix(
        ctx.accounts.cypher_group.key,
        ctx.accounts.cypher_user.key,
        ctx.accounts.user_signer.key,
        ctx.accounts.dex_market.key,
        ctx.accounts.open_orders.key,
        ctx.accounts.init_oo_authority.key,
    );
    invoke_signed(
        &ix,
        &ToAccountInfos::to_account_infos(&ctx),
        ctx.signer_seeds,
    )?;
    Ok(())
}

pub fn close_open_orders<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, CloseOpenOrdersCpi<'info>>,
) -> Result<()> {
    let ix = close_open_orders_ix(
        ctx.accounts.cypher_group.key,
        ctx.accounts.cypher_user.key,
        ctx.accounts.user_signer.key,
        ctx.accounts.dex_market.key,
        ctx.accounts.open_orders.key,
    );
    invoke_signed(
        &ix,
        &ToAccountInfos::to_account_infos(&ctx),
        ctx.signer_seeds,
    )?;
    Ok(())
}

pub fn new_order_v3<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, NewOrderV3Cpi<'info>>,
    data: NewOrderInstructionV3,
) -> Result<()> {
    let ix = new_order_v3_ix(
        ctx.accounts.cypher_group.key,
        ctx.accounts.vault_signer.key,
        ctx.accounts.price_history.key,
        ctx.accounts.cypher_user.key,
        ctx.accounts.user_signer.key,
        ctx.accounts.c_asset_mint.key,
        ctx.accounts.cypher_c_asset_vault.key,
        ctx.accounts.cypher_pc_vault.key,
        ctx.accounts.NoOpNewOrderV3dex.market.key,
        ctx.accounts.NoOpNewOrderV3dex.open_orders.key,
        ctx.accounts.NoOpNewOrderV3dex.req_q.key,
        ctx.accounts.NoOpNewOrderV3dex.event_q.key,
        ctx.accounts.NoOpNewOrderV3dex.bids.key,
        ctx.accounts.NoOpNewOrderV3dex.asks.key,
        ctx.accounts.NoOpNewOrderV3dex.coin_vault.key,
        ctx.accounts.NoOpNewOrderV3dex.pc_vault.key,
        ctx.accounts.NoOpNewOrderV3dex.vault_signer.key,
        data,
    );
    invoke_signed(
        &ix,
        &ToAccountInfos::to_account_infos(&ctx),
        ctx.signer_seeds,
    )?;
    Ok(())
}

pub fn cancel_order_v2<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, CancelOrderCpi<'info>>,
    data: CancelOrderInstructionV2,
) -> Result<()> {
    let ix = cancel_order_v2_ix(
        ctx.accounts.cypher_group.key,
        ctx.accounts.vault_signer.key,
        ctx.accounts.cypher_user.key,
        ctx.accounts.user_signer.key,
        ctx.accounts.c_asset_mint.key,
        ctx.accounts.cypher_c_asset_vault.key,
        ctx.accounts.cypher_pc_vault.key,
        ctx.accounts.NoOpCancelOrderdex.market.key,
        ctx.accounts.NoOpCancelOrderdex.prune_authority.key,
        ctx.accounts.NoOpCancelOrderdex.open_orders.key,
        ctx.accounts.NoOpCancelOrderdex.event_q.key,
        ctx.accounts.NoOpCancelOrderdex.bids.key,
        ctx.accounts.NoOpCancelOrderdex.asks.key,
        ctx.accounts.NoOpCancelOrderdex.coin_vault.key,
        ctx.accounts.NoOpCancelOrderdex.pc_vault.key,
        ctx.accounts.NoOpCancelOrderdex.vault_signer.key,
        data,
    );
    invoke_signed(
        &ix,
        &ToAccountInfos::to_account_infos(&ctx),
        ctx.signer_seeds,
    )?;
    Ok(())
}

pub fn cancel_order_by_client_id_v2<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, CancelOrderCpi<'info>>,
    client_order_id: u64,
) -> Result<()> {
    let ix = cancel_order_by_client_id_v2_ix(
        ctx.accounts.cypher_group.key,
        ctx.accounts.vault_signer.key,
        ctx.accounts.cypher_user.key,
        ctx.accounts.user_signer.key,
        ctx.accounts.c_asset_mint.key,
        ctx.accounts.cypher_c_asset_vault.key,
        ctx.accounts.cypher_pc_vault.key,
        ctx.accounts.NoOpCancelOrderdex.market.key,
        ctx.accounts.NoOpCancelOrderdex.prune_authority.key,
        ctx.accounts.NoOpCancelOrderdex.open_orders.key,
        ctx.accounts.NoOpCancelOrderdex.event_q.key,
        ctx.accounts.NoOpCancelOrderdex.bids.key,
        ctx.accounts.NoOpCancelOrderdex.asks.key,
        ctx.accounts.NoOpCancelOrderdex.coin_vault.key,
        ctx.accounts.NoOpCancelOrderdex.pc_vault.key,
        ctx.accounts.NoOpCancelOrderdex.vault_signer.key,
        client_order_id,
    );
    invoke_signed(
        &ix,
        &ToAccountInfos::to_account_infos(&ctx),
        ctx.signer_seeds,
    )?;
    Ok(())
}

pub fn settle_funds<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, SettleFundsCpi<'info>>,
) -> Result<()> {
    let ix = settle_funds_ix(
        ctx.accounts.cypher_group.key,
        ctx.accounts.vault_signer.key,
        ctx.accounts.cypher_user.key,
        ctx.accounts.user_signer.key,
        ctx.accounts.c_asset_mint.key,
        ctx.accounts.cypher_c_asset_vault.key,
        ctx.accounts.cypher_pc_vault.key,
        ctx.accounts.NoOpSettleFundsdex.market.key,
        ctx.accounts.NoOpSettleFundsdex.open_orders.key,
        ctx.accounts.NoOpSettleFundsdex.coin_vault.key,
        ctx.accounts.NoOpSettleFundsdex.pc_vault.key,
        ctx.accounts.NoOpSettleFundsdex.vault_signer.key,
    );
    invoke_signed(
        &ix,
        &ToAccountInfos::to_account_infos(&ctx),
        ctx.signer_seeds,
    )?;
    Ok(())
}
