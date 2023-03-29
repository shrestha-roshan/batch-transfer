use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction::transfer;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{
        transfer_checked as token_transfer, Mint, Token, TokenAccount,
        TransferChecked as TokenTransfer,
    },
};
use spl_token::instruction::transfer_checked as spl_transfer;
declare_id!("FddwD1WwcAN5XfR3bzD9jW1wHs4Bg1jz7YXTMicshzZp");

// seeds
const BATCH_SEED: &[u8] = b"transfer-batch";

#[program]
pub mod batch_transfer {

    use anchor_lang::solana_program::program::{invoke, invoke_signed};

    use super::*;

    pub fn deposit_sol(ctx: Context<DepositSol>, amount: u64) -> Result<()> {
        let authority = ctx.accounts.authority.to_account_info();
        let batch_vault = ctx.accounts.batch_vault.to_account_info();

        let ix = transfer(&authority.key(), &batch_vault.key(), amount);

        _ = invoke(&ix, &[authority, batch_vault]);

        Ok(())
    }

    pub fn deposit_token(ctx: Context<DepositToken>, amount: u64) -> Result<()> {
        let token_program = ctx.accounts.token_program.to_account_info();
        let from = ctx.accounts.from.to_account_info();
        let to = ctx.accounts.batch_vault_token_account.to_account_info();
        let authority = ctx.accounts.authority.to_account_info();
        let mint = &ctx.accounts.mint;

        let accounts = TokenTransfer {
            from,
            to,
            authority,
            mint: mint.to_account_info(),
        };

        let ctx = CpiContext::new(token_program, accounts);

        token_transfer(ctx, amount, mint.decimals)?;
        Ok(())
    }

    pub fn batch_sol_transfer(ctx: Context<BatchSolTransfer>, amount: Vec<u64>) -> Result<()> {
        let from = &ctx.accounts.batch_vault;
        let receivers = ctx.remaining_accounts.clone();

        for (i, receiver) in receivers.iter().enumerate() {
            if i < receivers.len() - 1 {
                let minimum_balance_for_rent_exemption =
                    Rent::get()?.minimum_balance(from.data_len());
                msg!(
                    "minimum rent exemption {:?}",
                    minimum_balance_for_rent_exemption
                );

                let transferable_amount = from
                    .lamports()
                    .checked_sub(minimum_balance_for_rent_exemption)
                    .unwrap_or_else(|| panic!("Error in deducting rent exemption"));

                require_gt!(transferable_amount, amount[i]);

                **receiver.lamports.borrow_mut() = receiver
                    .lamports()
                    .checked_add(amount[i])
                    .unwrap_or_else(|| panic!("Error in adding transfer amount"));

                **from.lamports.borrow_mut() = from
                    .lamports()
                    .checked_sub(amount[i])
                    .unwrap_or_else(|| panic!("Error in substracting transfer amount"));
            }
        }
        Ok(())
    }

    pub fn batch_token_transfer(ctx: Context<BatchTokenTransfer>, amount: Vec<u64>) -> Result<()> {
        let from = &ctx.accounts.batch_vault_token_account;
        let receivers = ctx.remaining_accounts;
        let mint = &ctx.accounts.mint;

        let mut ixs = vec![];
        for (i, receiver) in receivers.iter().enumerate() {
            if i < receivers.len() - 4 {
                let ix = spl_transfer(
                    &spl_token::id(),
                    &from.key(),
                    &mint.key(),
                    &receiver.key,
                    &ctx.accounts.batch_vault.key(),
                    &[],
                    amount[i],
                    mint.decimals,
                )?;
                ixs.push(ix);
            }
        }
        msg!("invoking");
        let authority_key = ctx.accounts.from_authority.key();
        let bump = ctx
            .bumps
            .get("batch_vault")
            .unwrap_or_else(|| panic!("Bump is missing."))
            .to_be_bytes();
        let signer_seeds: &[&[&[u8]]] = &[&[BATCH_SEED, authority_key.as_ref(), bump.as_ref()]];
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
        seeds = [BATCH_SEED, from_authority.key().as_ref()],
        bump,
    )]
    ///CHECK::
    pub batch_vault: AccountInfo<'info>,
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
        space = 0,
        seeds = [BATCH_SEED, from_authority.key().as_ref()],
        bump,
    )]
    ///CHECK::
    pub batch_vault: AccountInfo<'info>,
    ///CHECK::
    #[account(mut)]
    pub from_authority: AccountInfo<'info>,
    #[account(
        associated_token::mint = mint,
        associated_token::authority = batch_vault,
    )]
    pub batch_vault_token_account: Account<'info, TokenAccount>,
    pub mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositSol<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init_if_needed,
        payer = authority,
        seeds = [BATCH_SEED, authority.key().as_ref()],
        bump,
        space = 0
    )]
    /// CHECK::
    pub batch_vault: AccountInfo<'info>,

    pub rent: Sysvar<'info, Rent>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositToken<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = authority,
    )]
    pub from: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = authority,
        seeds = [BATCH_SEED, authority.key().as_ref()],
        bump,
        space = 0
    )]
    /// CHECK::
    pub batch_vault: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = batch_vault,
    )]
    pub batch_vault_token_account: Box<Account<'info, TokenAccount>>,

    pub mint: Box<Account<'info, Mint>>,

    pub rent: Sysvar<'info, Rent>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}
