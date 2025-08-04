#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use anchor_lang::prelude::borsh;
    use litesvm::LiteSVM;
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_program::{entrypoint::ProgramResult, system_program};
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;

    #[test]
    fn test_initialize_ring() -> ProgramResult {
        let mut svm = LiteSVM::new();
        let payer_kp = Keypair::new();
        let payer_pk = payer_kp.pubkey();
        println!("Payer address: {}", payer_pk);
        let program_id = Pubkey::from_str(&crate::ID.to_string()).unwrap();
        println!("Program id: {}", program_id);
        let bytes = include_bytes!("../../../target/deploy/sring.so");
        svm.add_program(program_id.clone(), bytes);
        // svm.with_coverage(
        //     vec![(program_id, "native_app".into())],
        //     vec![],
        //     payer_kp.insecure_clone(),
        // )
        // .unwrap();

        svm.airdrop(&payer_pk, 100000000000).unwrap();
        let recent_blockhash = svm.latest_blockhash();

        // Derive expected PDA and bump (example seeds)
        let seeds = &[b"sring", payer_pk.as_ref()];
        let (pda_pubkey, _bump) = Pubkey::find_program_address(seeds, &program_id);

        let ix_accounts = vec![
            // ORDER IS IMPORTANT!
            AccountMeta::new(payer_pk, true),
            AccountMeta::new(pda_pubkey, false), // even if not created already we must pass it!
            AccountMeta::new(system_program::ID, false),
        ];
        // discriminator only here - no args!
        let ix_data = vec![183, 129, 68, 92, 121, 234, 98, 108];
        let instructions = [Instruction::new_with_bytes(
            program_id,
            &ix_data,
            ix_accounts.clone(),
        )];

        let trans = Transaction::new_signed_with_payer(
            &instructions[..],
            Some(&payer_pk),
            &[&payer_kp],
            recent_blockhash,
        );

        let res = svm.send_transaction(trans.clone()).unwrap();
        println!("sring's initialize_ring -> {}", res.pretty_logs());

        let frames_num_to_init = 16;
        // Now add some frame slots.
        for i in 0..frames_num_to_init {
            println!("iterating! {}", i);
            svm.expire_blockhash();
            let recent_blockhash = svm.latest_blockhash();

            // discriminator + a borshed count...
            let mut ix_data = vec![110, 185, 160, 221, 234, 187, 242, 234];
            let ix_data = {
                let count = i as u64 + 1;
                let data = borsh::to_vec(&count)?;
                // println!("empty vec borshed -> {:?}", data);
                ix_data.extend_from_slice(&data);
                ix_data
            };
            let instructions = [Instruction::new_with_bytes(
                program_id,
                &ix_data,
                ix_accounts.clone(),
            )];

            let trans = Transaction::new_signed_with_payer(
                &instructions[..],
                Some(&payer_pk),
                &[&payer_kp],
                recent_blockhash,
            );

            let res = svm.send_transaction(trans.clone()).unwrap();
            println!("sring's add_frame_slot -> {}", res.pretty_logs());
        }

        Ok(())
    }
}
