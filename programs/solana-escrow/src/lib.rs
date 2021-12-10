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

    pub fn make(
        ctx: Context<Make>,
        escrowed_maker_tokens_bump: u8,
        offer_maker_amount: u64,  // this works for decimals
        offer_taker_amount: u64
    ) -> ProgramResult {
        /*
        - Verify constraints (should mostly be done in anchor context though)
        - Update new escrow account to store what mint + amt we are offering and want to receive
        - Transfer user's funds into vault account (escrowed_maker_tokens)
        */

        let offer = &mut ctx.accounts.offer;
        offer.maker = ctx.accounts.offer_maker.key();
        offer.taker_mint = ctx.accounts.taker_mint.key();
        offer.taker_amount = offer_taker_amount;
        offer.escrowed_maker_tokens_bump = escrowed_maker_tokens_bump;

        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),  // call the token program
                anchor_spl::token::Transfer {  // pass new transfer object
                    from: ctx.accounts.offer_makers_maker_tokens.to_account_info(),
                    to: ctx.accounts.escrowed_maker_tokens.to_account_info(),
                    // The offer_maker had to sign from the client in order for this to work
                    authority: ctx.accounts.offer_maker.to_account_info(),
                }
            ),
            offer_maker_amount,
        );

        Ok(())
    }

    pub fn accept(ctx: Context<Accept>) -> ProgramResult {

        //  TODO: (takers_taker_tokens.amount >= offer.escrowed_maker_tokens.taker_amount)
        //        (takers_taker_tokens.mint == offer.escrowed_maker_tokens.taker_mint.key()) &&
        Ok(())
    }

    pub fn cancel(ctx: Context<Cancel>) -> ProgramResult {
        Ok(())
    }
}

#[account]
pub struct Offer { 
    pub maker: Pubkey,
    pub taker_mint: Pubkey,
    pub taker_amount: u64,
    pub escrowed_maker_tokens_bump: u8,  // we can derive token account from this. We store it so client doesn't have to keep passing it in (convenience)
    // - In Accept and Cancel, we need to know the details of the vault account called `escrowed_maker_tokens` (e.g. amount).
    //   But we don't store the public key of the vault account (escrowed_maker_tokens) in the Offer account
    // - We can't store the whole account inside an account
    // - And we can't store just the public key because we won't be able to fetch the account details
    //    because Solana requires us to specify all the accounts we need to read from up front
    // - So we cannot store the value_account. That's okay because we can derive it from offer's address + seeds
}

#[derive(Accounts)]
#[instruction(escrowed_maker_tokens_bump: u8, offer_maker_amount: u64, offer_taker_amount: u64)]
pub struct Make<'info> {
    /*
    - offer: Need to store the offer details somewhere
    - offer_maker: Who is making the offer?
    - maker_mint: What's the mint type offer? (This needs to be in ctx because it's an account)
    - taker_mint: What's the desired mint type? (Amounts not in ctx because they're not accounts)
    - rent, system_program, token_program required by anchor
    */
     // offer: accounts can only be owned by program, so program owns Offer.
     // Nobody has "authority" over the account. Authority is a higher-level concept in solana, and you have to bake it into your program if you want it
     // There's no need for authority; the program can write to the offer account whenever it wants (based on its instructions)
    #[account(init, payer = offer_maker, space = 8 + 32 + 32 + 8 + 1)]
    pub offer: Account<'info, Offer>, 
    #[account(mut)]  // mut because paying for stuff
    pub offer_maker: Signer<'info>,
    #[account(mut, constraint = (
        (offer_makers_maker_tokens.mint == maker_mint.key()) &&
        (offer_makers_maker_tokens.owner == *offer_maker.key) &&
        (offer_makers_maker_tokens.amount >= maker_mint_amt)
    ))]
    pub offer_makers_maker_tokens: Account<'info, TokenAccount>,

    /*
        Initialize a new TokenAccount using CPI.
        anchor_spl::token isn't a separate program. It's just helper functions.
    */
    #[account(
        // seeds: Why do we need to pass seeds in if we already have the PDA to sign the txn? 
        // It tells anchor how to sign for the address. We need the address to sign because it's being initialized (i.e. its data is being changed).
        // Solana needs to verify that the PDA actually is from the program's ID, and it needs the seeds to help it calculate that.
        // (Given just the PDA and program ID, you can't necessarily guess the seeds and bump). -- i THINK!
        // init: initialize new account at location of PDA (seeds & bump combo). Why init at PDA? 
        // So that it's located at a memorable address. Otherwise we have to store keypair (inconvenient)
        init,  // all accounts created by anchor are rent-exempt by default
        payer = offer_maker, 
        seeds = [offer.key().as_ref()],
        bump = escrowed_maker_tokens_bump,
        token::mint = maker_mint,
        token::authority = escrowed_maker_tokens, // Needs to be the PDA of the program so that users don't have control over it
    )]
    pub escrowed_maker_tokens: Account<'info, TokenAccount>,

    pub maker_mint: Account<'info, Mint>,
    pub taker_mint: Account<'info, Mint>,
    
    // Why do we need to pass in the `rent` program if we don't actually use it?
    // spl_token functions require this input parameter. We don't need to pass it in explicitly
    // because, under the hood, anchor passes it in when calling spl_token functions.
    // If we weren't using spl_token and wanted to do rent-related stuff (e.g. is_exempt),
    // the modern way is to just to do Rent::get() or something similar.
    pub token_program: Program<'info, Token>,  // Automatically checks to make sure this value equals spl_token::id()
    pub system_program: Program<'info, System>,  // not pub system_program: AccountInfo<'info> because that doesn't check 
    pub rent: Sysvar<'info, Rent>, 
    
}

#[derive(Accounts)]
pub struct Accept<'info> {
    #[account(
        mut,
        constraint = offer.
        // BUG: Offer is a user-owned account. What would happen then?
    )]
    pub offer: Account<'info, Offer>, 
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(mut, constraint = (
        takers_taker_tokens.owner == *taker.key
    ))]
    pub takers_taker_tokens: Account<'info, TokenAccount>,
    // #[account(address = offer.taker_mint)]
    pub taker_mint: Account<'info, Mint>
    pub token_program: Program<'info, Token>,  // Automatically checks to make sure this value equals spl_token::id()
}

#[derive(Accounts)]
pub struct Cancel {

}

