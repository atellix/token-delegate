use anchor_lang::prelude::*;
use anchor_spl::token::{ self, TokenAccount, Transfer, Approve, Token };

declare_id!("TKDLLzzmBD7Rwbz6PS4XDFDr5w4ApSBRNC4wninsX7M");

#[program]
pub mod token_delegate {
    use super::*;

    // Link SPL token account to the token-delegate program
    pub fn delegate_link(ctx: Context<DelegateLink>,
        inp_amount: u64,
    ) -> anchor_lang::Result<()> {
        let cpi_accounts = Approve {
            to: ctx.accounts.token_account.to_account_info(),
            delegate: ctx.accounts.delegate_root.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::approve(cpi_ctx, inp_amount)
    }

    // Approve a sub-delegate for later SPL token transfers, and optionally link SPL token account to the token-delegate program
    pub fn delegate_approve(ctx: Context<DelegateApprove>,
        inp_link_token: bool,
        inp_link_amount: u64,
        inp_allowance_amount: u64,
    ) -> anchor_lang::Result<()> {
        // Optionally link token
        if inp_link_token {
            let cpi_accounts = Approve {
                to: ctx.accounts.token_account.to_account_info(),
                delegate: ctx.accounts.delegate_root.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            token::approve(cpi_ctx, inp_link_amount)?;
        }
        // Ensure signer is token account owner
        require_keys_eq!(ctx.accounts.token_account.owner, ctx.accounts.owner.key(), ErrorCode::InvalidTokenAccountOwner);
        let allowance = &mut ctx.accounts.allowance;
        allowance.owner = *ctx.accounts.owner.to_account_info().key;
        allowance.token_account = *ctx.accounts.token_account.to_account_info().key;
        allowance.delegate = *ctx.accounts.delegate.to_account_info().key;
        allowance.amount = inp_allowance_amount;
        Ok(())
    }

    // Perform a delegated transfer and update the allowance
    pub fn delegate_transfer(ctx: Context<DelegateTransfer>,
        inp_amount: u64,
    ) -> anchor_lang::Result<()> {
        //msg!("Transfer amount: {}", inp_amount.to_string());
        let allowance = &mut ctx.accounts.allowance;
        require_keys_eq!(allowance.token_account, ctx.accounts.from.key(), ErrorCode::InvalidTokenAccount);
        if inp_amount > 0 {
            //msg!("Begin: {}", ald.amount.to_string());
            let diff = allowance.amount.checked_sub(inp_amount);
            if diff.is_some() {
                // Perform transfer
                allowance.amount = diff.unwrap();
                //msg!("Allowance: {}", ald.amount.to_string());
                let (_pk, root_bump) = Pubkey::find_program_address(
                    &[ctx.program_id.as_ref()],
                    ctx.program_id
                );
                let seeds = &[
                    ctx.program_id.as_ref(),
                    &[root_bump]
                ];
                let signer = &[&seeds[..]];
                let cpi_accounts = Transfer {
                    from: ctx.accounts.from.to_account_info(),
                    to: ctx.accounts.to.to_account_info(),
                    authority: ctx.accounts.delegate_root.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_program.to_account_info();
                let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
                token::transfer(cpi_ctx, inp_amount)?;
            } else {
                msg!("Delegate transfer amount: {} exceeds allowance: {}", inp_amount.to_string(), allowance.amount.to_string());
                return Err(ErrorCode::AllowanceExceeded.into());
            }
        }
        Ok(())
    }

    // Update the delegate allowance amount
    pub fn delegate_update_amount(ctx: Context<DelegateUpdateAmount>,
        inp_amount: u64,
    ) -> anchor_lang::Result<()> {
        ctx.accounts.allowance.amount = inp_amount;
        Ok(())
    }

    // Close the delegate allowance and recover the storage fee
    pub fn delegate_close(_ctx: Context<DelegateClose>) -> anchor_lang::Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct DelegateLink<'info> {
    pub owner: Signer<'info>,
    /// CHECK: ok
    #[account(seeds = [program_id.as_ref()], bump)]
    pub delegate_root: UncheckedAccount<'info>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct DelegateApprove<'info> {
    #[account(init_if_needed, seeds = [token_account.key().as_ref(), owner.key().as_ref(), delegate.key().as_ref()], bump, payer = allowance_payer, space = 112)]
    pub allowance: Account<'info, DelegateAllowance>,
    #[account(mut)]
    pub allowance_payer: Signer<'info>,
    pub owner: Signer<'info>,
    /// CHECK: ok
    pub delegate: UncheckedAccount<'info>,
    /// CHECK: ok
    #[account(seeds = [program_id.as_ref()], bump)]
    pub delegate_root: UncheckedAccount<'info>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DelegateTransfer<'info> {
    #[account(mut, has_one = delegate)]
    pub allowance: Account<'info, DelegateAllowance>,
    pub delegate: Signer<'info>,
    /// CHECK: ok
    #[account(seeds = [program_id.as_ref()], bump)]
    pub delegate_root: UncheckedAccount<'info>,
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct DelegateUpdateAmount<'info> {
    #[account(mut, has_one = owner)]
    pub allowance: Account<'info, DelegateAllowance>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct DelegateClose<'info> {
    #[account(mut, has_one = owner, close = fee_recipient)]
    pub allowance: Account<'info, DelegateAllowance>,
    pub owner: Signer<'info>,
    /// CHECK: ok
    #[account(mut)]
    pub fee_recipient: UncheckedAccount<'info>,
}

#[account]
#[derive(Default)]
pub struct DelegateAllowance {
    pub owner: Pubkey,                  // The owner of the allowance (must be same as the owner of the token account)
    pub token_account: Pubkey,          // The token account for the allowance
    pub delegate: Pubkey,               // The delegate granted an allowance of tokens to transfer (typically the root PDA of another program)
    pub amount: u64,                    // The amount of tokens for the allowance (same decimals as underlying token)
}
// LEN: 8 + 32 + 32 + 32 + 8 = 112

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid Token Account")]
    InvalidTokenAccount,
    #[msg("Invalid token account owner")]
    InvalidTokenAccountOwner,
    #[msg("Allowance exceeded")]
    AllowanceExceeded,
}
