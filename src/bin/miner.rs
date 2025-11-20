/// Hourcoin Miner Client
///
/// Standalone miner binary that connects to a validator and mines blocks

use blockchainlib::MinerClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Hourcoin Miner Client ===\n");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    let miner_id = if args.len() > 1 {
        args[1].clone()
    } else {
        format!("miner_{}", rand::random::<u32>())
    };

    let validator_address = if args.len() > 2 {
        args[2].clone()
    } else {
        "127.0.0.1:8080".to_string()
    };

    let reward_address = if args.len() > 3 {
        args[3].clone()
    } else {
        miner_id.clone()
    };

    println!("Configuration:");
    println!("  Miner ID: {}", miner_id);
    println!("  Validator: {}", validator_address);
    println!("  Reward Address: {}", reward_address);
    println!();

    // Create miner client
    let client = MinerClient::new(miner_id.clone(), validator_address.clone());

    // Get initial round info
    println!("Connecting to validator...");
    match client.get_round_info().await {
        Ok(info) => {
            println!("✓ Connected to validator");
            println!("\nCurrent Round Info:");
            println!("  Tonce: {}", info.tonce.unwrap_or(0));
            println!("  Challenge time remaining: {} seconds", info.challenge_seconds_remaining);
            println!("  Difficulty: {}", info.difficulty);
            println!("  Attempted miners: {}", info.attempted_miners);
            println!("  Active lockouts: {}", info.active_lockouts);
            println!();
        }
        Err(e) => {
            eprintln!("✗ Failed to connect to validator: {}", e);
            eprintln!("Make sure the validator is running on {}", validator_address);
            return Ok(());
        }
    }

    // Check lockout status
    match client.check_lockout().await {
        Ok((is_locked, seconds_remaining)) => {
            if is_locked {
                println!("⏳ Currently in lockout period: {} seconds remaining\n", seconds_remaining);
            } else {
                println!("✓ Ready to mine!\n");
            }
        }
        Err(e) => {
            eprintln!("Warning: Could not check lockout status: {}", e);
        }
    }

    // Start mining
    println!("Starting continuous mining...");
    println!("Press Ctrl+C to stop\n");

    let difficulty = 0x00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF; // This will be queried from validator
    client.start_mining(
        vec![0; 32], // Genesis prev hash
        0,           // Starting index
        difficulty,
        &reward_address,
    ).await?;

    Ok(())
}
