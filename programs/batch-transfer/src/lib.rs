use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction::transfer;
use anchor_spl::{
    associated_token::AssociatedToken, 
    token::{
        TokenAccount, 
        Mint, 
        Token, 
        Transfer as TokenTransfer, 
        transfer as token_transfer
    }, 
}; 

declare_id!("FddwD1WwcAN5XfR3bzD9jW1wHs4Bg1jz7YXTMicshzZp");

#[program]
pub mod batch_transfer {

    use super::*;

    pub fn batch_sol_transfer(ctx: Context<BatchSolTransfer>, amount: Vec<u64>) -> Result<()> {
        msg!("Batch transfer solana token");
        let from = &ctx.accounts.from;
        let receivers = ctx.remaining_accounts.clone();

        let mut ixs = vec![];
        for (i, receiver) in receivers.iter().enumerate() {
            if i < receivers.len() - 2 {
                let ix = transfer(&from.key, &receiver.key, amount[i]);
                ixs.push(ix);
            }
        }
        msg!("invoking");
        for ix in ixs.iter() {
            invoke(&ix, receivers)?;
        }
        Ok(())
    }

    pub fn deposit_token(ctx: Context<DepositToken>, amount: u64) -> Result<()> {
        let token_program = ctx.accounts.token_program.to_account_info();
        let from = ctx.accounts.from.to_account_info();
        let to = ctx.accounts.vault.to_account_info();
        let authority =ctx.accounts.authority.to_account_info();

        ctx.accounts.ledger.authority = ctx.accounts.authority.key();

        let accounts = TokenTransfer {
            from,
            to,
            authority 
        };

        let ctx = CpiContext::new(token_program, accounts);
        
        token_transfer(ctx, amount)
    }


    pub fn spl_transfer(ctx: Context<SplTransfer>, amount: u64) -> Result<()> {
        let token_program = ctx.accounts.token_program.to_account_info();
        let from = ctx.accounts.vault.to_account_info();
        let to = ctx.accounts.to.to_account_info();
        let ledger = ctx.accounts.ledger.to_account_info();
        
        let vault_amount = ctx.accounts.vault.amount;
        assert!(vault_amount > amount, "Vault amount is less than transfer amount");

        let accounts = TokenTransfer {
            from,
            to,
            authority: ledger 
        };

        let authority_key = ctx.accounts.authority.key();
        let bump = ctx.bumps
            .get("ledger")
            .unwrap_or_else(|| panic!("Bump is missing."))
            .to_be_bytes();
        let signer_seeds: &[&[&[u8]]] = &[&[b"Ledger", authority_key.as_ref(), bump.as_ref()]];

        let ctx = CpiContext::new_with_signer(
            token_program,
            accounts,
            signer_seeds
        );
        
        token_transfer(ctx, amount)
    }
}


#[derive(Clone, Accounts)]
pub struct BatchSolTransfer<'info> {
    #[account(signer)]
    /// CHECK:
    pub from: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
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
    pub from : Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = authority,
        seeds = [b"Ledger", authority.key().as_ref()],
        bump,
        space = 8 + 32
    )]
    pub ledger: Box<Account<'info, RegistrationLedger>>,
    
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = ledger,
    )]
    pub vault: Box<Account<'info, TokenAccount>>,
    
    pub mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
    
    pub system_program: Program<'info, System>,
    
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[account]
pub struct RegistrationLedger {
    authority: Pubkey,
}


#[derive(Accounts)]
pub struct SplTransfer<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        seeds = [b"Ledger", authority.key().as_ref()],
        bump,
    )]
    pub ledger: Box<Account<'info, RegistrationLedger>>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = ledger,
    )]
    pub vault: Box<Account<'info, TokenAccount>>,
    
    /// CHECK: receiver's account
    pub to_owner: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = to_owner,
    )]
    pub to: Box<Account<'info, TokenAccount>>,

    pub mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}