#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::{Ipv4Addr, SocketAddrV4, UdpSocket},
        str::FromStr,
        sync::{Arc, Mutex},
    };

    use anchor_lang::prelude::borsh;
    use litesvm::LiteSVM;
    use solana_account::ReadableAccount;
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

            let _res = self
                .svm
                .send_transaction(trans.clone())
                .map_err(|e| e.err.to_string())?;
            // println!("sring's enqueue_frame -> {}", _res.pretty_logs());

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
            // println!("sring's dequeue_frame -> {}", res.pretty_logs());

            let frame = res.return_data.data;
            Ok(frame)
        }

        #[allow(dead_code)]
        pub fn get_pda_pk(&self) -> Pubkey {
            self.pda_pk.clone()
        }

        #[allow(dead_code)]
        pub fn inspect_data(&self) {
            eprintln!("{}", self.get_pda_pk());
            let account = self.svm.get_account(&self.get_pda_pk()).unwrap();
            let _ = std::fs::write("/tmp/data.txt", format!("{:#?}", account.data));
        }

        #[allow(dead_code)]
        pub fn inspect_lamports(&self) {
            eprintln!(
                "lamports left: {}",
                self.svm.get_account(&self.payer_pk).unwrap().lamports()
            );
        }
    }

    #[ignore]
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

    #[test]
    fn test_traffic() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        enum SenderJob {
            PacketEnqueued,
        }

        let tun_local_addr =
            Ipv4Addr::from_str(&std::env::var("TUN_LOCAL_ADDR").unwrap_or("2.1.1.1".into()))?;
        let tun_remote_addr =
            Ipv4Addr::from_str(&std::env::var("TUN_REMOTE_ADDR").unwrap_or("2.1.1.2".into()))?;
        let listen_addr_port = std::env::var("UDP_LISTEN").unwrap_or("0.0.0.0:2111".into());
        // let listen_addr = listen_addr_port.split(':').nth(0).expect("Wrong listen_addr_port format.");
        // let listen_port =listen_addr_port.split(':').nth(1).expect("Wrong listen_addr_port format.");
        let dst_addr_port = std::env::var("UDP_DST_ADDR").unwrap_or("192.168.0.99:2112".into());
        let dst_addr = dst_addr_port
            .split(':')
            .nth(0)
            .expect("Wrong dst_addr_port format.")
            .to_string();
        let dst_port: u16 = dst_addr_port
            .split(':')
            .nth(1)
            .expect("Wrong dst_addr_port format.")
            .parse()
            .expect("Wrong port");
        let tun_mtu = FRAME_LEN as u16;
        let mut config = tun::Configuration::default();
        config
            .address((
                tun_local_addr.octets()[0],
                tun_local_addr.octets()[1],
                tun_local_addr.octets()[2],
                tun_local_addr.octets()[3],
            ))
            .netmask((255, 255, 255, 0))
            .destination((
                tun_remote_addr.octets()[0],
                tun_remote_addr.octets()[1],
                tun_remote_addr.octets()[2],
                tun_remote_addr.octets()[3],
            ))
            .mtu(tun_mtu)
            .up();

        let udp_socket = Arc::new(UdpSocket::bind(listen_addr_port).unwrap());
        let dev = tun::create(&config)?;
        let (mut tun_reader, mut tun_writer) = dev.split();
        let (tx, rx) = crossbeam::channel::unbounded();
        let frame_ring = Arc::new(Mutex::new(FrameRing::new(&crate::ID)?));
        frame_ring.lock().unwrap().init()?;

        // UDP RX
        let _ = std::thread::spawn({
            let udp_socket = Arc::clone(&udp_socket);
            move || {
                let mut buf = [0; 2048];
                loop {
                    let amount = udp_socket.recv(&mut buf).unwrap();
                    let payload = &buf[..amount];
                    // println!("UDP RX: {:02x?}", payload);
                    tun_writer.write(&payload).unwrap();
                }
            }
        });

        // UDP TX
        let _ = std::thread::spawn({
            let frame_ring = Arc::clone(&frame_ring);
            let udp_socket = Arc::clone(&udp_socket);
            move || {
                let dst = SocketAddrV4::new(Ipv4Addr::from_str(&dst_addr).unwrap(), dst_port);
                for job in rx {
                    match job {
                        SenderJob::PacketEnqueued => {
                            if let Ok(frame) = frame_ring.lock().unwrap().dequeue_frame() {
                                // println!("{:02x?}", frame);
                                udp_socket.send_to(&frame, &dst).unwrap();
                            }
                        }
                    }
                }
            }
        });

        let mut buf = [0; 2048];
        loop {
            let amount = tun_reader.read(&mut buf)?;
            // println!("{:02x?}", &buf[..amount]);
            frame_ring.lock().unwrap().inspect_lamports();
            if let Ok(_) = frame_ring.lock().unwrap().enqueue_frame(&buf[..amount]) {
                tx.send(SenderJob::PacketEnqueued)?;
            }
        }
    }
}
