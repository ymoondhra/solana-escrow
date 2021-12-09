use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

declare_id!("B4BcAjTneKrCi6Gs9XabBeAGisu5GwTWAjPmYWgFCo2A");

// Best escrow guide: https://github.com/cqfd/quidproquo/blob/main/programs/quidproquo/src/lib.rs#L169

/*
TODO
- ADDITIONs
- TODOs

*/

/*
My Coding Approach
for instruction in ["make", "accept", "cancel"]:
    0. Brainstorm how instruction should work
    1. Create context for instruction
       a. Write the fields
       b. Write in constraints for those fields
    2. Write instruction
    3. Write test for that function
    4. Finish all three of those before moving onto the next function
*/

#[program]
pub mod solana_escrow {
    use super::*;

    pub fn make(ctx: Context<Make>, bump: u8, maker_mint_amt: u64, taker_mint_amt: u64) -> ProgramResult {
        /*
        - Verify constraints (should mostly be done in anchor context though)
        - Create new escrow account to store what mint + amt we are offering and want to receive
        - Create new token account to serve as vault, owned by PDA so it can sign transactions
        - Transfer user's funds into vault account
        */

        // TODO: Check if new account(s) have enough lamports to be rent-exempt
        // // if !rent.is_exempt()

        // let escrow_account = &mut ctx.accounts.escrow_account;
        // escrow_account.initiator = ctx.accounts.initiator.key();

        Ok(())
    }

    pub fn accept(ctx: Context<Accept>) -> ProgramResult {
        Ok(())
    }

    pub fn cancel(ctx: Context<Cancel>) -> ProgramResult {
        Ok(())
    }
}

#[account]
pub struct Offer { 
    pub offer_maker: Pubkey,
    pub maker_mint: Pubkey,
    pub taker_mint: Pubkey,
    pub taker_amount: u64,
    pub bump: u8,  // we can derive token account from this
    // - In Accept and Cancel, we need to know the details of the vault account called `escrowed_maker_tokens` (e.g. amount).
    //   But we don't store the public key of the vault account (escrowed_maker_tokens) in the Offer account
    // - We can't store the whole account inside an account
    // - And we can't store just the public key because we won't be able to fetch the account details
    //    because Solana requires us to specify all the accounts we need to read from up front
    // - So we cannot store the value_account. 
}

#[derive(Accounts)]
#[instruction(bump: u8, maker_mint_amt: u64, taker_mint_amt: u64)]
pub struct Make<'info> {
    /*
    - offer: Need to store the offer details somewhere
    - offer_maker: Who is making the offer?
    - maker_mint: What's the mint type offer? (This needs to be in ctx because it's an account)
    - taker_mint: What's the desired mint type? (Amounts not in ctx because they're not accounts)
    - rent, system_program, token_program required by anchor

    - offer_maker_token account: 
    - new_token_account: 
    
    */
    #[account(init, payer = offer_maker, space = 8 + 32 + 32 + 32 + 8 + 1)]
    pub offer: Account<'info, Offer>,  // TODO: Who owns this account?
    pub offer_maker: Signer<'info>,
    #[account(init, payer = offer_maker, constraint = (
        (offer_maker_token_account.mint == maker_mint.key()) // &&
        // () &&  // ADDITION: make sure they're the owner
        // ()     // ADDITION: make sure they have enough of maker_mint
        // (initiator_token_account.mint == mint_a.key()) &&
        // (initiator_token_account.amount >= mint_a_amt)
    ))]
    pub offer_maker_token_account: Account<'info, TokenAccount>,
    pub maker_mint: Account<'info, Mint>,
    pub taker_mint: Account<'info, Mint>,
    /*
        Initialize a new TokenAccount using CPI.
        anchor_spl::token isn't a separate program. It's just helper functions.
    */
    #[account(
        // seeds: Why do we need to pass seeds in if we already have the PDA to sign the txn? 
        // It tells anchor how to sign for the address. We need the address to sign because it's being initialized (i.e. its data is being changed).
        // Solana needs to verify that the PDA actually is from the program's ID, and it needs the seeds to help it calculate that.
        // (Given just the PDA and program ID, you can't necessarily guess the seeds and bump). -- i THINK!
        init, 
        seeds = [offer.key().as_ref()],  
        bump = bump,
        payer = offer_maker, 
        // token::mint = maker_mint,
        // token::authority = ,  // TODO: authority... PDA + can it also be the offer account?
                                 // TODO: Why is the the wallet address == PDA? I understand why it's the authority
    )]
    pub vault: Account<'info, TokenAccount>,
    
    // Why do we need to pass in the `rent` program if we don't actually use it?
    // spl_token functions require this input parameter. We don't need to pass it in explicitly
    // because, under the hood, anchor passes it in when calling spl_token functions.
    // If we weren't using spl_token and wanted to do rent-related stuff (e.g. is_exempt),
    // the modern way is to just to do Rent::get() or something similar.
    pub rent: Sysvar<'info, Rent>, 
    pub system_program: Program<'info, System>,  // not pub system_program: AccountInfo<'info> because that doesn't check 
    pub token_program: Program<'info, Token>,  // Automatically checks to make sure this value equals spl_token::id()
    // TODO: Does Program<'info, Token> just check format? or actually check if it is an account through Token program?
}

#[derive(Accounts)]
pub struct Accept {

}

#[derive(Accounts)]
pub struct Cancel {

}

