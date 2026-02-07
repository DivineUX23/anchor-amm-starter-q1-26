use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};
use constant_product_curve::{ConstantProduct, LiquidityPair};

use crate::{errors::AmmError, state::Config};


#[derive(Accounts)]
pub struct Swap <'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = user
    )]
    pub mint_x_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user
    )]
    pub mint_y_ata: Account<'info, TokenAccount>,

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
        seeds = [b"config", &config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump,
    )]
    pub mint_lp: Account<'info, Mint>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}


impl <'info> Swap <'info> {

    pub fn swap (&mut self, is_x: bool, amount: u64, min:u64) -> Result<()> {

        require!(self.config.locked == false, AmmError::PoolLocked);
        require!(amount > 0, AmmError::InvalidAmount);

        let mut curve = ConstantProduct::init(
            self.vault_x.amount, 
            self.vault_y.amount, 
            self.mint_lp.supply, 
            self.config.fee, 
            Some(6),
        )
        .unwrap();

        let trade_direction = match is_x {
            true => LiquidityPair::X,
            false => LiquidityPair::Y
        };

        let result = curve.swap(
            trade_direction, 
            amount, 
            min
        ).
        unwrap();

        self.give(is_x, result.deposit)?;
        self.receive(is_x, result.withdraw)

    }



    pub fn give (&self, is_x: bool, amount: u64) -> Result<()> {

        let (from, to) = match is_x {
            true => (
                self.mint_x_ata.to_account_info(),
                self.vault_x.to_account_info(),
            ),
            false => (
                self.mint_y_ata.to_account_info(),
                self.vault_y.to_account_info(),
            )
        };

        let account = Transfer{
            from,
            to,
            authority: self.user.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), account);

        transfer(cpi_ctx, amount)
    }



    pub fn receive (&self, is_x: bool, amount: u64) -> Result<()> {

        let signer_seeds : &[&[&[u8]]] = &[&[
            b"config",
             &self.config.seed.to_le_bytes(), 
             &[self.config.config_bump]
        ]];

        let (from, to) = match is_x {
            true => (
                self.vault_y.to_account_info(),
                self.mint_y_ata.to_account_info(),
            ),
            false => (
                self.vault_x.to_account_info(),
                self.mint_x_ata.to_account_info(),
            )
        };

        let account = Transfer{
            from,
            to,
            authority: self.config.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), account, signer_seeds);

        transfer(cpi_ctx, amount)
    }
}