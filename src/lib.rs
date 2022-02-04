use anchor_lang::prelude::*;
use anchor_lang::solana_program::{clock, program_option::COption, sysvar};
use anchor_spl::token::{self, Mint, Token, TokenAccount};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod coin_flip {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>, nonce: u8) -> ProgramResult {
        let coin_flip = &mut ctx.accounts.coin_flip;
        coin_flip.win_returns = 90;
        coin_flip.token_mint = ctx.accounts.token_mint.key();
        coin_flip.token_vault = ctx.accounts.token_vault.key();
        coin_flip.nonce = nonce;

        Ok(())
    }

    pub fn flip(ctx: Context<Flip>, amount: u64) -> ProgramResult {
        if amount == 0 {
            return Err(ErrorCode::AmountMustBeGreaterThanZero.into());
        }

        let coin_flip = &mut ctx.accounts.coin_flip;
        let c = clock::Clock::get().unwrap();

        // Transfer tokens into the token vault.
        {
            let cpi_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.stake_from_account.to_account_info(),
                    to: ctx.accounts.token_vault.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            );
            token::transfer(cpi_ctx, amount)?;
        }

        if (c.unix_timestamp % 2) == 0 {
            if ctx.accounts.token_vault.amount < ((amount * (coin_flip.win_returns as u64))/100) {
                msg!("Congratulations, You won! Sry, we didn't have enough reward to gib you. So, we'll gib you all the remaining reward in the vault");

                // Transfer tokens from the vault to user vault.
                {
                    let seeds = &[coin_flip.to_account_info().key.as_ref(), &[coin_flip.nonce]];
                    let pool_signer = &[&seeds[..]];

                    let cpi_ctx = CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        token::Transfer {
                            from: ctx.accounts.token_vault.to_account_info(),
                            to: ctx.accounts.stake_from_account.to_account_info(),
                            authority: ctx.accounts.pool_signer.to_account_info(),
                        },
                        pool_signer,
                    );
                    token::transfer(cpi_ctx, amount)?;
                }
            } else {
                // Transfer tokens from the vault to user vault.
                {
                    let seeds = &[coin_flip.to_account_info().key.as_ref(), &[coin_flip.nonce]];
                    let pool_signer = &[&seeds[..]];

                    let cpi_ctx = CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        token::Transfer {
                            from: ctx.accounts.token_vault.to_account_info(),
                            to: ctx.accounts.stake_from_account.to_account_info(),
                            authority: ctx.accounts.pool_signer.to_account_info(),
                        },
                        pool_signer,
                    );
                    token::transfer(cpi_ctx, amount * (100 + coin_flip.win_returns as u64)/100)?;
                }

                msg!("Congratulations, You won!");
            }
        } else {
            msg!("Sorry, You lost!");
        }

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct Initialize<'info> {
    #[account(
        zero
    )]
    pub coin_flip: Account<'info, CoinFlip>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,

    pub token_mint: Account<'info, Mint>,
    #[account(
        constraint = token_vault.mint == token_mint.key(),
        constraint = token_vault.owner == pool_signer.key()
    )]
    pub token_vault: Account<'info, TokenAccount>,

    #[account(
        seeds = [
            coin_flip.to_account_info().key.as_ref()
        ],
        bump = nonce,
    )]
    pub pool_signer: UncheckedAccount<'info>,
}

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct Flip<'info> {
    #[account(
        mut,
        has_one = token_vault
    )]
    pub coin_flip: Account<'info, CoinFlip>,

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        constraint = token_vault.owner == pool_signer.key()
    )]
    pub token_vault: Account<'info, TokenAccount>,

    // the token account of the user
    #[account(mut)]
    pub stake_from_account: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [
            coin_flip.to_account_info().key.as_ref()
        ],
        bump = nonce,
    )]
    pub pool_signer: UncheckedAccount<'info>,

    // Misc.
    token_program: Program<'info, Token>,
}

#[account]
#[derive(Default)]
pub struct CoinFlip {
    pub win_returns: u8,
    pub token_mint: Pubkey,
    pub token_vault: Pubkey,
    pub nonce: u8,
}

#[error]
pub enum ErrorCode {
    #[msg("Amount must be greater than zero.")]
    AmountMustBeGreaterThanZero,
}