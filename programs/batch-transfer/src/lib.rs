use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction::transfer;
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
}
#[derive(Clone, Accounts)]

pub struct BatchSolTransfer<'info> {
    #[account(signer)]
    /// CHECK:
    pub from: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
