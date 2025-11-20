/// Hourcoin Validator Server
///
/// Standalone validator binary that runs the Proof of Time consensus
/// and accepts connections from miner clients

use blockchainlib::ValidatorServer;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Hourcoin Validator Server ===\n");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    let address = if args.len() > 1 {
        args[1].clone()
    } else {
        "127.0.0.1:8080".to_string()
    };

    let difficulty = if args.len() > 2 {
        u128::from_str_radix(&args[2].trim_start_matches("0x"), 16)
            .unwrap_or(0x00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF)
    } else {
        0x00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
    };

    println!("Configuration:");
    println!("  Address: {}", address);
    println!("  Difficulty: 0x{:X}", difficulty);
    println!();

    // Create and start the validator server
    let mut server = ValidatorServer::new(difficulty, address);

    println!("Starting Proof of Time consensus...\n");

    server.start().await?;

    Ok(())
}
