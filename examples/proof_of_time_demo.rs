/// Proof of Time Consensus Demo
///
/// This example demonstrates Hourcoin's proof of time consensus mechanism:
/// 1. Validator maintains the blockchain and enforces tonce challenges
/// 2. Miners submit blocks that pass the tonce divisibility test
/// 3. Accepted miners enter a 1-hour lockout period (miner sacrifice)
/// 4. After 60 seconds, tonce reduces to 1 (race condition)

use blockchainlib::*;

fn main() {
    println!("=== Hourcoin Proof of Time Demo ===\n");

    // Set up the validator with medium difficulty
    let difficulty = 0x00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
    let mut validator = Validator::new(difficulty);

    println!("1. Validator initialized with difficulty: 0x{:X}", difficulty);
    println!("   Block count: {}\n", validator.get_block_count());

    // Start the first mining round
    validator.start_new_round();
    let round_info = validator.get_round_info();

    println!("2. New mining round started:");
    println!("   Tonce: {}", round_info.tonce.unwrap_or(0));
    println!("   Challenge time remaining: {} seconds", round_info.challenge_seconds_remaining);
    println!("   Attempted miners: {}\n", round_info.attempted_miners);

    // Create and mine the genesis block
    println!("3. Miner 'Alice' mining genesis block...");
    let timestamp = now();

    // Find a timestamp that satisfies the tonce challenge
    let tonce = validator.get_current_tonce().unwrap();
    let valid_timestamp = find_valid_timestamp(tonce, timestamp, 10000)
        .expect("Failed to find valid timestamp");

    println!("   Found valid timestamp for tonce {}: {}", tonce, valid_timestamp);

    let coinbase = transaction::Transaction {
        inputs: vec![],
        outputs: vec![transaction::Output {
            to_addr: "Alice".to_owned(),
            value: 2.0,
            timestamp: valid_timestamp,
        }],
    };

    let mut genesis_block = Block::new(0, valid_timestamp, vec![0; 32], vec![coinbase]);
    genesis_block.mine(difficulty);

    println!("   Block mined! Hash: {}", hex::encode(&genesis_block.hash));
    println!("   Nonce: {}\n", genesis_block.nonce);

    // Submit the block
    let result = validator.validate_block_submission(genesis_block.clone(), "Alice".to_string());

    match result {
        ValidationResult::Accepted => {
            println!("4. ✓ Block ACCEPTED by validator!");
            println!("   Blockchain now has {} block(s)", validator.get_block_count());
            println!("   Alice entered 1-hour lockout period\n");
        }
        _ => {
            println!("4. ✗ Block REJECTED: {:?}\n", result);
            return;
        }
    }

    // Check Alice's lockout status
    let lockout_remaining = validator.get_miner_lockout_remaining("Alice");
    println!("5. Miner lockout status:");
    println!("   Alice: {} seconds remaining", lockout_remaining);
    println!("   Is locked out: {}\n", validator.is_miner_in_lockout("Alice"));

    // Try to have Alice mine again (should be rejected)
    println!("6. Alice attempts to mine again immediately...");
    let timestamp2 = now();
    let coinbase2 = transaction::Transaction {
        inputs: vec![],
        outputs: vec![transaction::Output {
            to_addr: "Alice".to_owned(),
            value: 2.0,
            timestamp: timestamp2,
        }],
    };

    let prev_hash = genesis_block.hash.clone();
    let mut block2 = Block::new(1, timestamp2, prev_hash.clone(), vec![coinbase2]);
    block2.mine(difficulty);

    let result2 = validator.validate_block_submission(block2, "Alice".to_string());

    match result2 {
        ValidationResult::RejectedMinerInLockout => {
            println!("   ✓ Correctly rejected: Miner is in lockout period\n");
        }
        _ => {
            println!("   ✗ Unexpected result: {:?}\n", result2);
        }
    }

    // Have Bob mine the next block
    println!("7. Miner 'Bob' mining next block...");
    let round_info2 = validator.get_round_info();
    let tonce2 = round_info2.tonce.unwrap();

    println!("   New tonce: {}", tonce2);
    println!("   Challenge time remaining: {} seconds", round_info2.challenge_seconds_remaining);

    let timestamp3 = now();
    let valid_timestamp3 = find_valid_timestamp(tonce2, timestamp3, 10000)
        .expect("Failed to find valid timestamp");

    let coinbase3 = transaction::Transaction {
        inputs: vec![],
        outputs: vec![transaction::Output {
            to_addr: "Bob".to_owned(),
            value: 2.0,
            timestamp: valid_timestamp3,
        }],
    };

    let mut block3 = Block::new(1, valid_timestamp3, prev_hash, vec![coinbase3]);
    block3.mine(difficulty);

    println!("   Block mined! Hash: {}", hex::encode(&block3.hash));

    let result3 = validator.validate_block_submission(block3, "Bob".to_string());

    match result3 {
        ValidationResult::Accepted => {
            println!("   ✓ Block ACCEPTED by validator!");
            println!("   Blockchain now has {} block(s)", validator.get_block_count());
            println!("   Bob entered 1-hour lockout period\n");
        }
        _ => {
            println!("   ✗ Block REJECTED: {:?}\n", result3);
        }
    }

    // Show final state
    println!("8. Final validator state:");
    let final_round = validator.get_round_info();
    println!("   Total blocks: {}", validator.get_block_count());
    println!("   Active lockouts: {}", final_round.active_lockouts);
    println!("   Current tonce: {}", final_round.tonce.unwrap_or(0));
    println!("   Challenge time remaining: {} seconds", final_round.challenge_seconds_remaining);

    println!("\n=== Demo Complete ===");
    println!("\nKey Proof of Time Features Demonstrated:");
    println!("✓ Tonce-based mining challenges");
    println!("✓ Timestamp validation");
    println!("✓ Miner sacrifice protocol (1-hour lockout)");
    println!("✓ Prevention of rapid re-mining");
    println!("✓ Fair mining rounds with randomized difficulty");
}
