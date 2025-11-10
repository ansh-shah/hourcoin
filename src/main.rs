use blockchainlib::*;
use rand::Rng; // used to generate random u128 numbers for timestamp examples

fn main() {
	println!("=== Hourcoin: Proof of Time Blockchain ===\n");

	let difficulty = 0x0000FFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
    let mut rng = rand::thread_rng();

	println!("Part 1: Basic Blockchain Demo\n");
	println!("Mining genesis block with traditional PoW...");

	// example of genesis block with coinbase transaction
	let mut genesis_block = Block::new(0, now(), vec![0; 32], vec![Transaction {
																		inputs: vec![],
																		outputs: vec![
																			transaction::Output{
																				value: 1.5,
																				to_addr: "Alice".to_owned(),
                                                                                timestamp: now()
																			},
																			transaction::Output{
																				value: 0.5,
																				to_addr: "Bob".to_owned(),
                                                                                timestamp: now()
																			}]}],);

	genesis_block.mine(difficulty);
	println!("✓ Mined genesis block: {:?}", &genesis_block);

	let last_hash = genesis_block.hash.clone();

	let mut blockchain = Blockchain::new_with_diff(difficulty);

	blockchain.update_with_block(genesis_block).expect("Failed to add genesis block");

	println!("\nMining second block...");

	 let mut block = Block::new(1, now(), last_hash, vec![
        Transaction {
            inputs: vec![ ],
            outputs: vec![
                transaction::Output {
                    to_addr: "Chris".to_owned(),
                    value: 2.0,
                    timestamp: rng.gen(),
                },
            ],
        },
        Transaction {
            inputs: vec![
                blockchain.blocks[0].transactions[0].outputs[0].clone(),
            ],
            outputs: vec![
                transaction::Output {
                    to_addr: "Alice".to_owned(),
                    value: 0.25,
                    timestamp: rng.gen(),
                },
                transaction::Output {
                    to_addr: "Bob".to_owned(),
                    value: 0.5,
                    timestamp: rng.gen(),
                },
            ],
        },
    ],);

	block.mine(blockchain.get_difficulty());

    println!("✓ Mined block: {:?}", &block);

    blockchain.update_with_block(block).expect("Failed to add block");

	println!("\n✓ Blockchain now has {} blocks", blockchain.blocks.len());

	// Demonstrate proof of time system
	println!("\n\nPart 2: Proof of Time Consensus Demo\n");

	let mut validator = Validator::new(difficulty);
	validator.start_new_round();

	println!("Validator initialized with proof of time consensus");
	let round_info = validator.get_round_info();
	println!("Tonce for this round: {}", round_info.tonce.unwrap_or(0));
	println!("Challenge duration: 60 seconds");
	println!("Miner lockout period: 1 hour\n");

	// Simulate miner finding valid timestamp
	let timestamp = now();
	let tonce = validator.get_current_tonce().unwrap();

	println!("Miner 'Alice' searching for valid timestamp with tonce {}...", tonce);

	if let Some(valid_timestamp) = find_valid_timestamp(tonce, timestamp, 5000) {
		println!("✓ Found valid timestamp: {}", valid_timestamp);

		// Create and mine block
		let coinbase = Transaction {
			inputs: vec![],
			outputs: vec![transaction::Output {
				to_addr: "Alice".to_owned(),
				value: 2.0,
				timestamp: valid_timestamp,
			}],
		};

		let mut new_block = Block::new(0, valid_timestamp, vec![0; 32], vec![coinbase]);
		new_block.mine(difficulty);

		println!("✓ Block mined with hash: {}", hex::encode(&new_block.hash[..8]));

		// Submit to validator
		let result = validator.validate_block_submission(new_block, "Alice".to_string());

		match result {
			ValidationResult::Accepted => {
				println!("✓ Block ACCEPTED by validator!");
				println!("✓ Alice is now in 1-hour lockout period");
				println!("  Lockout remaining: {} seconds", validator.get_miner_lockout_remaining("Alice"));
			}
			_ => {
				println!("✗ Block rejected: {:?}", result);
			}
		}
	} else {
		println!("✗ Could not find valid timestamp within attempts");
	}

	println!("\n=== Summary ===");
	println!("Traditional PoW blockchain: {} blocks", blockchain.blocks.len());
	println!("Proof of Time validator: {} blocks", validator.get_block_count());
	println!("\nProof of Time adds:");
	println!("• Time-based mining challenges (tonce)");
	println!("• Miner sacrifice protocol (hourly lockouts)");
	println!("• Fair, time-valued consensus");
}
