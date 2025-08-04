use anchor_lang::prelude::*;

mod tests;

declare_id!("52rK34hDp4374vyuc2Psp2951rEpGwj7UpisGVeHjdWW");

pub const FRAME_LEN: u64 = 1024;
pub const INITIAL_FRAMES_NUM: u64 = 0;

#[derive(InitSpace)]
#[account]
pub struct SRing {
    pub current_idx: u64,
    pub next_idx: u64,
    #[max_len(INITIAL_FRAMES_NUM)]
    pub frames: Vec<[u8; FRAME_LEN as usize]>,
}

#[program]
pub mod sring {
    use super::*;

    pub fn initialize_ring(ctx: Context<InitializeRing>) -> Result<()> {
        msg!("Greetings from: {:?} - initialize_ring", ctx.program_id);
        let ring_account = &mut ctx.accounts.ring_account;
        ring_account.current_idx = 0;
        ring_account.next_idx = 0;

        Ok(())
    }

    pub fn add_frame_slot(ctx: Context<AddFrameSlot>, _count: u64) -> Result<()> {
        msg!("Greetings from: {:?} - add_frame_slot", ctx.program_id);
        let ring_account = &mut ctx.accounts.ring_account;
        let frame_num = ring_account.frames.len() + 1;

        msg!("Setting up {} frame", frame_num);
        ring_account.frames.push([0u8; FRAME_LEN as usize]);

        msg!(
            "Current vec len: {}",
            ring_account.frames.len() * FRAME_LEN as usize
        );
        Ok(())
    }
}

#[derive(Accounts)]
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

#[derive(Accounts)]
#[instruction(count: u64)]
pub struct AddFrameSlot<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    // The to-be realloc'd PDA.
    #[account(mut, seeds = [b"sring", owner.key.as_ref()], bump,
        realloc = 8 + SRing::INIT_SPACE + (count as usize) * FRAME_LEN as usize,
        realloc::payer = owner,
        realloc::zero = false,
    )]
    pub ring_account: Account<'info, SRing>,

    // System program is required for account manipulation.
    pub system_program: Program<'info, System>,
}
