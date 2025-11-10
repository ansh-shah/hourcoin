/// Miner client for Hourcoin
///
/// Connects to a validator server, mines blocks, and submits them

use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::{Block, now, find_valid_timestamp, transaction};
use super::protocol::*;

/// Miner client that connects to a validator
pub struct MinerClient {
    miner_id: String,
    validator_address: String,
}

impl MinerClient {
    /// Create a new miner client
    pub fn new(miner_id: String, validator_address: String) -> Self {
        MinerClient {
            miner_id,
            validator_address,
        }
    }

    /// Connect to the validator
    async fn connect(&self) -> Result<TcpStream, Box<dyn std::error::Error>> {
        let stream = TcpStream::connect(&self.validator_address).await?;
        Ok(stream)
    }

    /// Send a message to the validator and receive a response
    async fn send_message(
        &self,
        stream: &mut TcpStream,
        message: MinerMessage,
    ) -> Result<ValidatorMessage, Box<dyn std::error::Error>> {
        // Serialize message
        let message_json = serde_json::to_vec(&message)?;
        let len_bytes = (message_json.len() as u32).to_be_bytes();

        // Send message
        stream.write_all(&len_bytes).await?;
        stream.write_all(&message_json).await?;
        stream.flush().await?;

        // Read response length
        let mut len_buffer = [0u8; 4];
        stream.read_exact(&mut len_buffer).await?;
        let response_len = u32::from_be_bytes(len_buffer) as usize;

        // Read response
        let mut response_buffer = vec![0u8; response_len];
        stream.read_exact(&mut response_buffer).await?;

        let response: ValidatorMessage = serde_json::from_slice(&response_buffer)?;
        Ok(response)
    }

    /// Get current round information from validator
    pub async fn get_round_info(&self) -> Result<RoundInfoData, Box<dyn std::error::Error>> {
        let mut stream = self.connect().await?;

        let message = MinerMessage::GetRoundInfo {
            miner_id: self.miner_id.clone(),
        };

        let response = self.send_message(&mut stream, message).await?;

        match response {
            ValidatorMessage::RoundInfo(info) => Ok(info),
            ValidatorMessage::Error { message } => Err(message.into()),
            _ => Err("Unexpected response".into()),
        }
    }

    /// Check lockout status
    pub async fn check_lockout(&self) -> Result<(bool, u64), Box<dyn std::error::Error>> {
        let mut stream = self.connect().await?;

        let message = MinerMessage::CheckLockout {
            miner_id: self.miner_id.clone(),
        };

        let response = self.send_message(&mut stream, message).await?;

        match response {
            ValidatorMessage::LockoutStatus { is_locked, seconds_remaining } => {
                Ok((is_locked, seconds_remaining))
            }
            ValidatorMessage::Error { message } => Err(message.into()),
            _ => Err("Unexpected response".into()),
        }
    }

    /// Mine and submit a block
    pub async fn mine_and_submit(
        &self,
        prev_hash: Vec<u8>,
        index: u32,
        difficulty: u128,
        reward_address: &str,
    ) -> Result<ValidatorMessage, Box<dyn std::error::Error>> {
        println!("Mining block #{}...", index);

        // Get round info to know the tonce
        let round_info = self.get_round_info().await?;

        if let Some(tonce) = round_info.tonce {
            println!("  Tonce: {}", tonce);
            println!("  Challenge time remaining: {} seconds", round_info.challenge_seconds_remaining);

            // Find valid timestamp
            let start_time = now();
            let valid_timestamp = find_valid_timestamp(tonce, start_time, 100000)
                .ok_or("Failed to find valid timestamp")?;

            println!("  Found valid timestamp: {}", valid_timestamp);

            // Create coinbase transaction
            let coinbase = transaction::Transaction {
                inputs: vec![],
                outputs: vec![transaction::Output {
                    to_addr: reward_address.to_owned(),
                    value: 2.0,
                    timestamp: valid_timestamp,
                }],
            };

            // Create and mine block
            let mut block = Block::new(index, valid_timestamp, prev_hash, vec![coinbase]);
            block.mine(difficulty);

            println!("  ✓ Block mined! Hash: {}", hex::encode(&block.hash[..8]));
            println!("  Nonce: {}", block.nonce);

            // Submit block
            println!("  Submitting to validator...");

            let mut stream = self.connect().await?;

            let message = MinerMessage::SubmitBlock {
                miner_id: self.miner_id.clone(),
                block: BlockData::from_block(&block),
            };

            let response = self.send_message(&mut stream, message).await?;
            Ok(response)
        } else {
            Err("No tonce available".into())
        }
    }

    /// Start continuous mining (mine until lockout, wait, repeat)
    pub async fn start_mining(
        &self,
        initial_prev_hash: Vec<u8>,
        initial_index: u32,
        difficulty: u128,
        reward_address: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut prev_hash = initial_prev_hash;
        let mut index = initial_index;

        loop {
            // Check if we're in lockout
            let (is_locked, seconds_remaining) = self.check_lockout().await?;

            if is_locked {
                println!("\n⏳ In lockout period. Waiting {} seconds...\n", seconds_remaining);
                tokio::time::sleep(tokio::time::Duration::from_secs(seconds_remaining + 1)).await;
                continue;
            }

            // Mine and submit
            match self.mine_and_submit(prev_hash.clone(), index, difficulty, reward_address).await {
                Ok(ValidatorMessage::BlockResult { result, message }) => {
                    match result {
                        BlockResultType::Accepted => {
                            println!("  ✓ {}", message);
                            // For demonstration, increment index (in real scenario, get from validator)
                            index += 1;
                            // Note: In production, we'd query the validator for the latest block hash
                        }
                        _ => {
                            println!("  ✗ Block rejected: {}", message);
                            // Wait a bit before retrying
                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        }
                    }
                }
                Ok(msg) => {
                    println!("  Unexpected response: {:?}", msg);
                }
                Err(e) => {
                    eprintln!("  Error: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }

            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_miner_client_creation() {
        let client = MinerClient::new("test_miner".to_string(), "127.0.0.1:8080".to_string());
        assert_eq!(client.miner_id, "test_miner");
        assert_eq!(client.validator_address, "127.0.0.1:8080");
    }
}
