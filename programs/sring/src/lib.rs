use anchor_lang::prelude::*;

declare_id!("52rK34hDp4374vyuc2Psp2951rEpGwj7UpisGVeHjdWW");

#[program]
pub mod sring {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
