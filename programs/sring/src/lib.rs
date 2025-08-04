use anchor_lang::prelude::*;

mod tests;

declare_id!("52rK34hDp4374vyuc2Psp2951rEpGwj7UpisGVeHjdWW");

pub const FRAME_LEN: u64 = 1024;
pub const FRAMES_NUM: u64 = 9;

#[derive(InitSpace)]
#[account]
pub struct SRing {
    pub current_idx: u64,
    pub next_idx: u64,
    #[max_len(FRAMES_NUM * FRAME_LEN)]
    pub frames: Vec<[u8; FRAME_LEN as usize]>,
}

#[program]
pub mod sring {
    use super::*;

    pub fn initialize_ring(ctx: Context<InitializeRing>) -> Result<()> {
        msg!("Greetings from: {:?} - initialize", ctx.program_id);
        let ring_account = &mut ctx.accounts.ring_account;
        ring_account.current_idx = 0;
        ring_account.next_idx = 0;
        for _ in 0..FRAMES_NUM {
            ring_account.frames.push([0u8; FRAME_LEN as usize]);
        }

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitializeRing<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    // The PDA being created, initialized with the derived space including the discriminator(8).
    #[account(
        init,
        payer = owner,
        space = 8 + SRing::INIT_SPACE,
        seeds = [b"sring", owner.key.as_ref()],
        bump,
    )]
    pub ring_account: Account<'info, SRing>,

    // System program is required for account creation.
    pub system_program: Program<'info, System>,
}
