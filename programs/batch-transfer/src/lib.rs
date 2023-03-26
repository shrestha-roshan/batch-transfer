use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction::transfer;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use spl_token::instruction::transfer as spl_transfer;
declare_id!("FddwD1WwcAN5XfR3bzD9jW1wHs4Bg1jz7YXTMicshzZp");

// seeds
const BATCH_TOKEN: &[u8] = b"batch-token";
const BATCH_SOL: &[u8] = b"batch-sol";

#[program]
pub mod batch_transfer {

    use anchor_lang::solana_program::program::invoke_signed;

    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
    pub fn initialize_sol(_ctx: Context<InitializeSolAccount>) -> Result<()> {
        Ok(())
    }

    pub fn batch_sol_transfer(ctx: Context<BatchSolTransfer>, amount: Vec<u64>) -> Result<()> {
        msg!("Batch transfer solana token");
        let from = &ctx.accounts.from;
        msg!("from: {:?}", from.key);
        let receivers = ctx.remaining_accounts.clone();

        let mut ixs = vec![];
        for (i, receiver) in receivers.iter().enumerate() {
            if i < receivers.len() - 1 {
                let ix = transfer(&from.key(), &receiver.key, amount[i]);
                ixs.push(ix);
            }
        }
        msg!("invoking");
        let authority_key = ctx.accounts.from_authority.key();
        let bump = ctx
            .bumps
            .get("from")
            .unwrap_or_else(|| panic!("Bump is missing."))
            .to_be_bytes();
        let signer_seeds: &[&[&[u8]]] = &[&[b"batch-test", authority_key.as_ref(), bump.as_ref()]];
        for ix in ixs.iter() {
            invoke_signed(&ix, receivers, signer_seeds)?;
        }
        Ok(())
    }

    pub fn batch_token_transfer(ctx: Context<BatchTokenTransfer>, amount: Vec<u64>) -> Result<()> {
        msg!("Batch transfer token");
        let from = &ctx.accounts.from;
        let receivers = ctx.remaining_accounts;

        let mut ixs = vec![];
        for (i, receiver) in receivers.iter().enumerate() {
            if i < receivers.len() - 3 {
                let ix = spl_transfer(
                    &spl_token::id(),
                    &from.key(),
                    &receiver.key,
                    &ctx.accounts.from_pda.key(),
                    &[],
                    amount[i],
                )?;
                ixs.push(ix);
            }
        }
        msg!("invoking");
        let authority_key = ctx.accounts.from_authority.key();
        let bump = ctx
            .bumps
            .get("from_pda")
            .unwrap_or_else(|| panic!("Bump is missing."))
            .to_be_bytes();
        let signer_seeds: &[&[&[u8]]] = &[&[BATCH_TOKEN, authority_key.as_ref(), bump.as_ref()]];
        for ix in ixs.iter() {
            //invoke
            invoke_signed(&ix, receivers, signer_seeds)?;
        }
        Ok(())
    }
}

#[derive(Clone, Accounts)]
pub struct BatchSolTransfer<'info> {
    #[account(
        seeds = [b"batch-test", from_authority.key().as_ref()],
        bump,
    )]
    ///CHECK::
    pub from: AccountInfo<'info>,
    ///CHECK::
    pub from_authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Clone, Accounts)]
pub struct BatchTokenTransfer<'info> {
    #[account(
        init_if_needed,
        payer = from_authority,
        space = 8,
        seeds = [BATCH_TOKEN, from_authority.key().as_ref()],
        bump,
    )]
    ///CHECK::
    pub from_pda: Box<Account<'info, BatchPda>>,
    ///CHECK::
    #[account(mut)]
    pub from_authority: AccountInfo<'info>,
    #[account(
        associated_token::mint = mint,
        associated_token::authority = from_pda,
    )]
    pub from: Account<'info, TokenAccount>,
    pub mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Clone, Accounts)]
pub struct Initialize<'info> {
    #[account(
        init_if_needed,
        seeds = [BATCH_TOKEN, from_authority.key().as_ref()],
        bump,
        payer = from_authority,
        space = 8
    )]
    pub from_pda: Box<Account<'info, BatchPda>>,
    #[account(
        init_if_needed,
        payer = from_authority,
        associated_token::mint = mint,
        associated_token::authority = from_pda,
    )]
    pub from_pda_token_account: Account<'info, TokenAccount>,
    #[account(signer, mut)]
    ///CHECK::
    pub from_authority: AccountInfo<'info>,
    pub mint: Box<Account<'info, Mint>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Clone, Accounts)]
pub struct InitializeSolAccount<'info> {
    #[account(
        init,
        seeds = [b"batch-test", from_authority.key().as_ref()],
        bump,
        payer = from_authority,
        space = 0
    )]
    ///CHECK::
    pub from_pda: AccountInfo<'info>,
    #[account(signer, mut)]
    ///CHECK::
    pub from_authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}
#[account]
pub struct BatchPda {}
