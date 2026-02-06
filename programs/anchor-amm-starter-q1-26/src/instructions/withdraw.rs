use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, 
    token::{Burn, Mint, MintTo, Token, TokenAccount, Transfer, burn, mint_to, transfer}};
use constant_product_curve::ConstantProduct;

use crate::{state::Config, errors::AmmError};

#[derive(Accounts)]
#[instruction(seed: u64)]

pub struct Withdraw <'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub mint_x_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub mint_y_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump,
    )]
    pub mint_lp: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
    )]
    pub user_lp: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config
    )]
    pub vault_x: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config
    )]
    pub vault_y: Account<'info, TokenAccount>,

    #[account(
        has_one = mint_x,
        has_one = mint_y,
        seeds = [b"config", seed.to_le_bytes().as_ref()],
        bump = config.config_bump
    )]
    pub config: Account<'info, Config>,
    
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}

impl <'info> Withdraw <'info> {


    pub fn withdraw (&mut self, min_x: u64, min_y: u64, amount: u64) -> Result<()> {
        require!(amount != 0, AmmError::InvalidAmount);
        require!(amount <= self.user_lp.amount, AmmError::InsufficientBalance);

        let amt_withdraw = ConstantProduct::xy_withdraw_amounts_from_l(
                self.vault_x.amount, 
                self.vault_y.amount, 
                self.mint_lp.supply,
                amount, 
                6,
            )
            .unwrap();

        require!(amt_withdraw.x >= min_x && amt_withdraw.y >= min_y, AmmError::SlippageExceeded);

        self.withdraw_token(true, amt_withdraw.x)?;
        self.withdraw_token(false, amt_withdraw.y)?;
        self.burn_lp_token(amount)
    }

    pub fn withdraw_token(&self, is_x: bool, amount: u64) -> Result<()> {

        let signer_seeds : &[&[&[u8]]] = &[&[
            b"config", 
            &self.config.seed.to_le_bytes(), 
            &[self.config.config_bump],
        ]];

        let(to, from) = match is_x {
            true => (
                self.mint_x_ata.to_account_info(),
                self.vault_x.to_account_info(),
            ),
            false => (
                self.mint_y_ata.to_account_info(),
                self.vault_y.to_account_info(),
            )
        };
        
        let cpi_accounts = Transfer{
            from,
            to,
            authority: self.config.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), cpi_accounts, signer_seeds);

        transfer(cpi_ctx, amount)
    }


    pub fn burn_lp_token (&self, amount: u64) -> Result<()> {

        let cpi_accounts = Burn{
            mint: self.mint_lp.to_account_info(),
            from: self.user_lp.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);

        burn(cpi_ctx, amount)
    }

}