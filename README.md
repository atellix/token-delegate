# token-delegate
Protocol to create multiple token delegates for SPL token accounts

A protocol to share one delegate with multiple programs.

## Import Statement
```rust
use token_delegate::{ self, cpi::accounts::{ DelegateApprove, DelegateTransfer } };
```

## Delegate Approve
### Enable a downstream program to transfer from a SPL token account 
```rust
let cpi_accounts = DelegateApprove {
    allowance: ctx.accounts.allowance.to_account_info(),            // PDA of Token Delegate Program: (token_account, owner, delegate)
    allowance_payer: ctx.accounts.user_key.to_account_info(),       // Payer to open the allowance account
    owner: ctx.accounts.this_user.to_account_info(),                // The owner of the main token account
    delegate: ctx.accounts.this_program.to_account_info(),          // The delegate account (probably a PDA for another program)
    delegate_root: ctx.accounts.delegate_root.to_account_info(),    // The PDA of the Token Delegate Program (the "delegate" for SPL token accounts)
    token_account: ctx.accounts.token_account.clone(),              // The SPL token account to link
    token_program: ctx.accounts.token_program.to_account_info(),    // SPL Token program
    system_program: ctx.accounts.system_program.to_account_info(),  // System program 
};
let cpi_program = ctx.accounts.delegate_program.to_account_info();
let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
token_delegate::cpi::delegate_approve(cpi_ctx, true, u64::MAX, u64::MAX)?;
```

## Delegate Transfer
### Transfer tokens with signature from delegate
```rust
// Root PDA of the current program (the delegate)
let root_pda_seeds = &[ctx.program_id.as_ref(), &[inp_root_nonce]];
let root_pda_signer = &[&root_pda_seeds[..]];

let cpi_accounts = DelegateTransfer {
    allowance: ctx.accounts.allowance.to_account_info(),            // PDA of Token Delegate Program: (token_account, owner, delegate)
    delegate: ctx.accounts.root_key.to_account_info(),              // The delegate account (probably a PDA for another program)
    delegate_root: ctx.accounts.delegate_root.to_account_info(),    // The PDA of the Token Delegate Program
    from: ctx.accounts.token_account.clone(),                       // The SPL token account that was delegated
    to: ctx.accounts.token_account_destination.clone(),             // The destination SPL token account for the transfer
    token_program: ctx.accounts.token_program.to_account_info(),    // SPL Token program
};
let cpi_program = ctx.accounts.delegate_program.to_account_info();
let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, root_pda_signer);
token_delegate::cpi::delegate_transfer(cpi_ctx, token_transfer)?;
```

