use anchor_lang::prelude::*;

mod errors;
mod instructions;
mod state;

use instructions::*;
declare_id!("3k6pmDkLLF7FBANTs1ddTCGpsxvDjx8UBc5tUryisKTG");

#[program]
pub mod anchor_amm_starter_q1_26 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, 
        seed: u64, 
        fee: u16, 
        authority: Option<Pubkey>,
    ) -> Result<()> {
        ctx.accounts.init(seed, fee, authority, ctx.bumps)
    }
}