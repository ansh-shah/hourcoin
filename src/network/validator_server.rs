/// Validator server for Hourcoin
///
/// Runs a TCP server that accepts connections from miners,
/// validates blocks, and maintains the blockchain

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::{Validator, ValidationResult};
use super::protocol::*;

/// Validator server that manages the proof of time consensus
pub struct ValidatorServer {
    validator: Arc<Mutex<Validator>>,
    address: String,
}

impl ValidatorServer {
    /// Create a new validator server
    pub fn new(difficulty: u128, address: String) -> Self {
        let validator = Validator::new(difficulty);
        ValidatorServer {
            validator: Arc::new(Mutex::new(validator)),
            address,
        }
    }

    /// Start the validator server
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize the first mining round
        {
            let mut validator = self.validator.lock().await;
            validator.start_new_round();
        }

        println!("Validator server starting on {}", self.address);
        println!("Waiting for miner connections...\n");

        let listener = TcpListener::bind(&self.address).await?;

        loop {
            let (socket, addr) = listener.accept().await?;
            println!("New connection from: {}", addr);

            let validator = Arc::clone(&self.validator);

            // Spawn a new task for each connection
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(socket, validator).await {
                    eprintln!("Error handling connection from {}: {}", addr, e);
                }
            });
        }
    }

    /// Handle a single miner connection
    async fn handle_connection(
        mut socket: TcpStream,
        validator: Arc<Mutex<Validator>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = vec![0u8; 1024 * 1024]; // 1MB buffer

        loop {
            // Read message length (4 bytes)
            let n = socket.read(&mut buffer[..4]).await?;
            if n == 0 {
                return Ok(()); // Connection closed
            }

            let msg_len = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;

            if msg_len > buffer.len() {
                return Err("Message too large".into());
            }

            // Read the actual message
            let n = socket.read_exact(&mut buffer[..msg_len]).await?;
            if n == 0 {
                return Ok(());
            }

            let request: MinerMessage = serde_json::from_slice(&buffer[..msg_len])?;

            let response = Self::process_message(request, &validator).await;

            // Send response
            let response_json = serde_json::to_vec(&response)?;
            let len_bytes = (response_json.len() as u32).to_be_bytes();

            socket.write_all(&len_bytes).await?;
            socket.write_all(&response_json).await?;
            socket.flush().await?;
        }
    }

    /// Process a message from a miner
    async fn process_message(
        message: MinerMessage,
        validator: &Arc<Mutex<Validator>>,
    ) -> ValidatorMessage {
        match message {
            MinerMessage::GetRoundInfo { miner_id } => {
                let validator = validator.lock().await;
                let round_info = validator.get_round_info();
                let difficulty = validator.get_difficulty();

                println!("Miner '{}' requested round info", miner_id);

                ValidatorMessage::RoundInfo(RoundInfoData::from_round_info(&round_info, difficulty))
            }

            MinerMessage::SubmitBlock { miner_id, block } => {
                println!("Miner '{}' submitting block #{}", miner_id, block.index);

                let block = match block.to_block() {
                    Ok(b) => b,
                    Err(e) => {
                        return ValidatorMessage::Error {
                            message: format!("Invalid block data: {}", e),
                        };
                    }
                };

                let mut validator = validator.lock().await;
                let result = validator.validate_block_submission(block, miner_id.clone());

                match &result {
                    ValidationResult::Accepted => {
                        println!("✓ Block ACCEPTED from miner '{}'", miner_id);
                        println!("  Miner entered 1-hour lockout");
                        println!("  Blockchain now has {} blocks\n", validator.get_block_count());

                        ValidatorMessage::BlockResult {
                            result: BlockResultType::from(&result),
                            message: "Block accepted! You are now in 1-hour lockout.".to_string(),
                        }
                    }
                    _ => {
                        println!("✗ Block REJECTED from miner '{}': {:?}", miner_id, result);

                        let message = match &result {
                            ValidationResult::RejectedMinerInLockout => {
                                format!("Miner in lockout. {} seconds remaining.",
                                    validator.get_miner_lockout_remaining(&miner_id))
                            }
                            ValidationResult::RejectedTonceChallenge => {
                                "Timestamp failed tonce challenge".to_string()
                            }
                            ValidationResult::RejectedInvalidTimestamp => {
                                "Invalid timestamp".to_string()
                            }
                            ValidationResult::RejectedMinerAlreadyAttempted => {
                                "Already attempted this round".to_string()
                            }
                            ValidationResult::RejectedBlockchainValidation(e) => {
                                format!("Blockchain validation failed: {}", e)
                            }
                            _ => format!("{:?}", result),
                        };

                        ValidatorMessage::BlockResult {
                            result: BlockResultType::from(&result),
                            message,
                        }
                    }
                }
            }

            MinerMessage::CheckLockout { miner_id } => {
                let validator = validator.lock().await;
                let is_locked = validator.is_miner_in_lockout(&miner_id);
                let seconds_remaining = validator.get_miner_lockout_remaining(&miner_id);

                ValidatorMessage::LockoutStatus {
                    is_locked,
                    seconds_remaining,
                }
            }

            MinerMessage::GetBlockchainInfo => {
                let validator = validator.lock().await;
                let block_count = validator.get_block_count();
                let difficulty = validator.get_difficulty();

                ValidatorMessage::BlockchainInfo {
                    block_count,
                    difficulty: format!("0x{:X}", difficulty),
                }
            }
        }
    }
}
