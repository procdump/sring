#[cfg(test)]
mod tests {
    use anchor_lang::prelude::borsh;
    use litesvm::LiteSVM;
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_program::system_program;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;

    use crate::{FRAMES_NUM, FRAME_LEN, FRAME_LEN_FIELD_LEN};

    pub struct FrameRing {
        pub svm: LiteSVM,
        pub payer_kp: Keypair,
        pub payer_pk: Pubkey,
        pub program_id: Pubkey,
        pub pda_pk: Pubkey,
        pub inited: bool,
    }

    impl FrameRing {
        pub fn new(program_id: &Pubkey) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
            let mut svm = LiteSVM::new();
            let payer_kp = Keypair::new();
            let payer_pk = payer_kp.pubkey();
            println!("Payer address: {}", payer_pk);
            let program_id = *program_id;
            println!("Program id: {}", program_id);
            // Derive expected PDA and bump (example seeds)
            let seeds = &[b"sring", payer_pk.as_ref()];
            let (pda_pk, _bump) = Pubkey::find_program_address(seeds, &program_id);

            let bytes = include_bytes!("../../../target/deploy/sring.so");
            svm.add_program(program_id.clone(), bytes);
            svm.airdrop(&payer_pk, 100000000000).unwrap();

            // svm.with_coverage(
            //     vec![(program_id, "native_app".into())],
            //     vec![],
            //     payer_kp.insecure_clone(),
            // )
            // .unwrap();

            Ok(Self {
                svm,
                payer_kp,
                payer_pk,
                program_id,
                inited: false,
                pda_pk,
            })
        }

        pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            if self.inited == true {
                return Err(String::from("Already inited!").into());
            }

            let ix_accounts = vec![
                // ORDER IS IMPORTANT!
                AccountMeta::new(self.payer_pk, true),
                AccountMeta::new(self.pda_pk, false), // even if not created already we must pass it!
                AccountMeta::new(system_program::ID, false),
            ];
            // discriminator only here - no args!
            let ix_data = vec![183, 129, 68, 92, 121, 234, 98, 108];
            let instructions = [Instruction::new_with_bytes(
                self.program_id,
                &ix_data,
                ix_accounts.clone(),
            )];

            self.svm.expire_blockhash();
            let recent_blockhash = self.svm.latest_blockhash();
            let trans = Transaction::new_signed_with_payer(
                &instructions[..],
                Some(&self.payer_pk),
                &[&self.payer_kp],
                recent_blockhash,
            );

            let res = self
                .svm
                .send_transaction(trans.clone())
                .map_err(|e| e.err.to_string())?;
            println!("sring's initialize_ring -> {}", res.pretty_logs());

            let frames_num_to_init = FRAMES_NUM;
            // Now add some frame slots.
            for i in 0..frames_num_to_init {
                println!("iterating! {}", i);
                self.svm.expire_blockhash();
                let recent_blockhash = self.svm.latest_blockhash();

                // discriminator + a borshed count...
                let mut ix_data = vec![150, 152, 61, 87, 21, 87, 223, 211];
                let ix_data = {
                    let count = i as u64 + 1;
                    let data = borsh::to_vec(&count)?;
                    // println!("empty vec borshed -> {:?}", data);
                    ix_data.extend_from_slice(&data);
                    ix_data
                };
                let instructions = [Instruction::new_with_bytes(
                    self.program_id,
                    &ix_data,
                    ix_accounts.clone(),
                )];

                let trans = Transaction::new_signed_with_payer(
                    &instructions[..],
                    Some(&self.payer_pk),
                    &[&self.payer_kp],
                    recent_blockhash,
                );

                let res = self.svm.send_transaction(trans.clone()).unwrap();
                println!("sring's add_frame_slot -> {}", res.pretty_logs());
            }

            self.inited = true;

            Ok(())
        }

        pub fn enqueue_frame(
            &mut self,
            frame: &[u8],
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let ix_accounts = vec![
                // ORDER IS IMPORTANT!
                AccountMeta::new(self.payer_pk, true),
                AccountMeta::new(self.pda_pk, false), // even if not created already we must pass it!
                AccountMeta::new(system_program::ID, false),
            ];

            let mut ix_data = vec![119, 252, 33, 107, 190, 193, 106, 137];
            let ix_data = {
                let frame = borsh::to_vec(frame)?;
                ix_data.extend_from_slice(&frame);
                ix_data
            };
            let instructions = [Instruction::new_with_bytes(
                self.program_id,
                &ix_data,
                ix_accounts.clone(),
            )];

            self.svm.expire_blockhash();
            let recent_blockhash = self.svm.latest_blockhash();
            let trans = Transaction::new_signed_with_payer(
                &instructions[..],
                Some(&self.payer_pk),
                &[&self.payer_kp],
                recent_blockhash,
            );

            let res = self
                .svm
                .send_transaction(trans.clone())
                .map_err(|e| e.err.to_string())?;
            println!("sring's enqueue_frame -> {}", res.pretty_logs());

            Ok(())
        }

        pub fn dequeue_frame(
            &mut self,
        ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
            let ix_accounts = vec![
                // ORDER IS IMPORTANT!
                AccountMeta::new(self.payer_pk, true),
                AccountMeta::new(self.pda_pk, false), // even if not created already we must pass it!
                AccountMeta::new(system_program::ID, false),
            ];

            let ix_data = vec![156, 73, 25, 215, 2, 39, 195, 43];
            let instructions = [Instruction::new_with_bytes(
                self.program_id,
                &ix_data,
                ix_accounts.clone(),
            )];

            self.svm.expire_blockhash();
            let recent_blockhash = self.svm.latest_blockhash();
            let trans = Transaction::new_signed_with_payer(
                &instructions[..],
                Some(&self.payer_pk),
                &[&self.payer_kp],
                recent_blockhash,
            );

            let res = self
                .svm
                .send_transaction(trans.clone())
                .map_err(|e| e.err.to_string())?;
            println!("sring's dequeue_frame -> {}", res.pretty_logs());

            let frame = res.return_data.data;
            Ok(frame)
        }

        pub fn get_pda_pk(&self) -> Pubkey {
            self.pda_pk.clone()
        }

        pub fn inspect_data(&self) {
            eprintln!("{}", self.get_pda_pk());
            let account = self.svm.get_account(&self.get_pda_pk()).unwrap();
            let _ = std::fs::write("/tmp/data.txt", format!("{:#?}", account.data));
        }
    }

    #[test]
    fn test_frame_ring() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut frame_ring = FrameRing::new(&crate::ID)?;
        frame_ring.init()?;

        let frame = [1u8; FRAME_LEN as usize - FRAME_LEN_FIELD_LEN as usize];
        for _ in 0..FRAMES_NUM {
            frame_ring.enqueue_frame(&frame)?;
        }

        // frame_ring.inspect_data();

        for _ in 0..FRAMES_NUM {
            let res = frame_ring.dequeue_frame()?;
            assert!(&res == &[1u8; FRAME_LEN as usize - FRAME_LEN_FIELD_LEN as usize]);
        }

        let frame = [1u8; FRAME_LEN as usize - FRAME_LEN_FIELD_LEN as usize];
        for _ in 0..FRAMES_NUM {
            frame_ring.enqueue_frame(&frame)?;
        }

        // frame_ring.inspect_data();

        for _ in 0..FRAMES_NUM {
            let res = frame_ring.dequeue_frame()?;
            assert!(&res == &[1u8; FRAME_LEN as usize - FRAME_LEN_FIELD_LEN as usize]);
        }

        let frame = [1u8; FRAME_LEN as usize - FRAME_LEN_FIELD_LEN as usize];
        for _ in 0..FRAMES_NUM * 8 {
            frame_ring.enqueue_frame(&frame)?;
            let res = frame_ring.dequeue_frame()?;
            assert!(&res == &[1u8; FRAME_LEN as usize - FRAME_LEN_FIELD_LEN as usize]);
        }

        Ok(())
    }
}
