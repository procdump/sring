use anchor_lang::prelude::*;

mod tests;

declare_id!("52rK34hDp4374vyuc2Psp2951rEpGwj7UpisGVeHjdWW");

pub const DISCRIMINATOR_LEN: u64 = 8;
pub const FRAME_LEN_FIELD_LEN: u64 = 8;
pub const FRAME_LEN: u64 = 1024 - FRAME_LEN_FIELD_LEN; // due to set_return_data's limit
pub const FRAMES_NUM: u64 = 512;

#[error_code]
pub enum SRingError {
    #[msg("No more slots possible to allocate")]
    SlotsFull,
    #[msg("The ring is full, cannot push more frames")]
    RingFull,
    #[msg("The ring is empty, cannot pop frames")]
    RingEmpty,
    #[msg("Invalid conversion")]
    InvalidConversion,
}

#[derive(InitSpace)]
#[account]
pub struct SRing {
    pub write_idx: u64,
    pub read_idx: u64,
    pub slots: u64,
    pub count: u64,
    pub capacity: u64,
    pub frame_size: u64,
}

#[program]
pub mod sring {
    use super::*;

    pub fn initialize_ring(ctx: Context<InitializeRing>) -> Result<()> {
        msg!("Greetings from: {:?} - initialize_ring", ctx.program_id);
        let ring_account = &mut ctx.accounts.ring_account;
        ring_account.write_idx = 0;
        ring_account.read_idx = 0;
        ring_account.count = 0;
        ring_account.slots = 0;
        ring_account.capacity = FRAMES_NUM;
        ring_account.frame_size = FRAME_LEN;

        Ok(())
    }

    pub fn add_frame_space(ctx: Context<AddFrameSlot>, count: u64) -> Result<()> {
        let ring_account = &mut ctx.accounts.ring_account;
        if ring_account.capacity < count {
            return Err(SRingError::SlotsFull.into());
        }

        ring_account.slots += 1;

        msg!("Greetings from: {:?} - add_frame_slot", ctx.program_id);
        msg!("Setting up {} frame", count);
        Ok(())
    }

    pub fn enqueue_frame(ctx: Context<EnqueueFrame>, frame: Vec<u8>) -> Result<()> {
        let ring_account = &mut ctx.accounts.ring_account;

        if ring_account.count == FRAMES_NUM {
            return Err(SRingError::RingFull.into());
        }

        let write_idx = ring_account.write_idx;
        let ai = &mut ring_account.to_account_info();
        let mut data = ai.try_borrow_mut_data()?;

        let frame_len = &mut data
            [DISCRIMINATOR_LEN as usize + SRing::INIT_SPACE + (write_idx * FRAME_LEN) as usize..];
        frame_len[..FRAME_LEN_FIELD_LEN as usize]
            .copy_from_slice(&(frame.len().to_le() as u64).to_le_bytes());

        let frame_slot = &mut data[DISCRIMINATOR_LEN as usize
            + SRing::INIT_SPACE
            + (write_idx * FRAME_LEN) as usize
            + FRAME_LEN_FIELD_LEN as usize..];
        frame_slot[..frame.len()].copy_from_slice(&frame);

        ring_account.write_idx = (ring_account.write_idx + 1) % FRAMES_NUM;
        ring_account.count += 1;

        Ok(())
    }

    pub fn dequeue_frame(ctx: Context<DequeueFrame>) -> Result<()> {
        // msg!("Greetings from dequeue_frame");
        let ring_account = &mut ctx.accounts.ring_account;
        if ring_account.count == 0 {
            return Err(SRingError::RingEmpty.into());
        }
        let read_idx = ring_account.read_idx;
        let ai = &mut ring_account.to_account_info();
        let data = ai.try_borrow_data()?;

        // msg!("Next frame offset: {}", DISCRIMINATOR_LEN as usize + SRing::INIT_SPACE + (read_idx * FRAME_LEN) as usize);
        let frame_len = &data
            [DISCRIMINATOR_LEN as usize + SRing::INIT_SPACE + (read_idx * FRAME_LEN) as usize..];
        let frame_len = &frame_len[..8];
        // msg!("frame_len raw: {:?}", frame_len);
        let frame_len: &[u8; 8] = frame_len
            .try_into()
            .map_err(|_| SRingError::InvalidConversion)?;
        let frame_len = u64::from_le_bytes(*frame_len);

        let frame_slot = &data[DISCRIMINATOR_LEN as usize
            + SRing::INIT_SPACE
            + (read_idx * FRAME_LEN) as usize
            + FRAME_LEN_FIELD_LEN as usize..];
        let frame_slot = &frame_slot[..frame_len as usize];
        // msg!("frame_slot_plus_len len: {}", frame_slot.len());

        anchor_lang::solana_program::program::set_return_data(frame_slot);
        ring_account.read_idx = (ring_account.read_idx + 1) % FRAMES_NUM;
        ring_account.count -= 1;

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
        space = DISCRIMINATOR_LEN as usize + SRing::INIT_SPACE,
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
        realloc = DISCRIMINATOR_LEN as usize + SRing::INIT_SPACE + (count as usize) * FRAME_LEN as usize,
        realloc::payer = owner,
        realloc::zero = false,
    )]
    pub ring_account: Account<'info, SRing>,

    // System program is required for account manipulation.
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EnqueueFrame<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    // The PDA holding the frames.
    #[account(mut, seeds = [b"sring", owner.key.as_ref()], bump)]
    pub ring_account: Account<'info, SRing>,

    // System program is required for account manipulation.
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DequeueFrame<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    // The PDA holding the frames.
    #[account(mut, seeds = [b"sring", owner.key.as_ref()], bump)]
    pub ring_account: Account<'info, SRing>,

    // System program is required for account manipulation.
    pub system_program: Program<'info, System>,
}
