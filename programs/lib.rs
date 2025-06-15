// programs/meme_launcher/src/lib.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use anchor_lang::solana_program::program::invoke;
use anchor_spl::associated_token::AssociatedToken;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod meme_launcher {
    use super::*;

    pub fn initialize_launch(
        ctx: Context<InitializeLaunch>,
        name: String,
        symbol: String,
        initial_supply: u64,
        curve_ratio: u64,
    ) -> Result<()> {
        let launch = &mut ctx.accounts.launch;
        launch.creator = ctx.accounts.creator.key();
        launch.mint = ctx.accounts.mint.key();
        launch.name = name;
        launch.symbol = symbol;
        launch.initial_supply = initial_supply;
        launch.curve_ratio = curve_ratio;
        launch.total_supply = initial_supply;
        launch.is_active = true;

        // Initialize token mint
        let mint_authority = &[&[b"mint_authority", launch.key().as_ref(), &[ctx.bumps.mint_authority]]];
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.creator_token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                mint_authority,
            ),
            initial_supply,
        )?;

        Ok(())
    }

    pub fn buy_tokens(
        ctx: Context<BuyTokens>,
        amount: u64,
    ) -> Result<()> {
        let launch = &mut ctx.accounts.launch;
        require!(launch.is_active, LaunchError::LaunchInactive);

        // Calculate price based on bonding curve
        let price = calculate_price(launch.total_supply, amount, launch.curve_ratio)?;
        
        // Transfer SOL from buyer to creator
        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.buyer.key(),
            &launch.creator,
            price,
        );
        invoke(
            &transfer_ix,
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.creator.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Mint tokens to buyer
        let mint_authority = &[&[b"mint_authority", launch.key().as_ref(), &[ctx.bumps.mint_authority]]];
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.buyer_token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                mint_authority,
            ),
            amount,
        )?;

        launch.total_supply += amount;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(name: String, symbol: String)]
pub struct InitializeLaunch<'info> {
    #[account(init, payer = creator, space = Launch::LEN)]
    pub launch: Account<'info, Launch>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    
    #[account(
        init,
        payer = creator,
        mint::decimals = 9,
        mint::authority = mint_authority,
    )]
    pub mint: Account<'info, Mint>,
    
    /// CHECK: PDA for mint authority
    #[account(
        seeds = [b"mint_authority", launch.key().as_ref()],
        bump
    )]
    pub mint_authority: UncheckedAccount<'info>,
    
    #[account(
        init,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = creator,
    )]
    pub creator_token_account: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BuyTokens<'info> {
    #[account(mut)]
    pub launch: Account<'info, Launch>,
    
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    /// CHECK: Creator account to receive SOL
    #[account(mut)]
    pub creator: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    /// CHECK: PDA for mint authority
    #[account(
        seeds = [b"mint_authority", launch.key().as_ref()],
        bump
    )]
    pub mint_authority: UncheckedAccount<'info>,
    
    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = mint,
        associated_token::authority = buyer,
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct Launch {
    pub creator: Pubkey,
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub initial_supply: u64,
    pub total_supply: u64,
    pub curve_ratio: u64,
    pub is_active: bool,
}

impl Launch {
    const LEN: usize = 8 + // discriminator
        32 + // creator
        32 + // mint
        32 + // name
        8 + // symbol
        8 + // initial_supply
        8 + // total_supply
        8 + // curve_ratio
        1; // is_active
}

#[error_code]
pub enum LaunchError {
    #[msg("Launch is not active")]
    LaunchInactive,
    #[msg("Invalid price calculation")]
    InvalidPriceCalculation,
}

// Helper function to calculate price based on bonding curve
fn calculate_price(current_supply: u64, amount: u64, curve_ratio: u64) -> Result<u64> {
    // Simple linear bonding curve: price = current_supply * curve_ratio * amount
    let price = (current_supply as u128)
        .checked_mul(curve_ratio as u128)
        .ok_or(LaunchError::InvalidPriceCalculation)?
        .checked_mul(amount as u128)
        .ok_or(LaunchError::InvalidPriceCalculation)?;
    
    Ok(price.try_into().map_err(|_| LaunchError::InvalidPriceCalculation)?)
}