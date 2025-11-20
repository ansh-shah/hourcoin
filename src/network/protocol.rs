/// Network protocol definitions for Hourcoin
///
/// Defines the message types exchanged between miners and validators

use serde::{Deserialize, Serialize};
use crate::{Block, ValidationResult, RoundInfo};

/// Messages sent from miner to validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MinerMessage {
    /// Miner requests current round information
    GetRoundInfo { miner_id: String },

    /// Miner submits a block for validation
    SubmitBlock {
        miner_id: String,
        block: BlockData,
    },

    /// Miner checks their lockout status
    CheckLockout { miner_id: String },

    /// Miner requests blockchain info
    GetBlockchainInfo,
}

/// Messages sent from validator to miner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidatorMessage {
    /// Round information response
    RoundInfo(RoundInfoData),

    /// Block submission result
    BlockResult {
        result: BlockResultType,
        message: String,
    },

    /// Lockout status response
    LockoutStatus {
        is_locked: bool,
        seconds_remaining: u64,
    },

    /// Blockchain information
    BlockchainInfo {
        block_count: usize,
        difficulty: String,
    },

    /// Error message
    Error { message: String },
}

/// Serializable block data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockData {
    pub index: u32,
    pub timestamp: u128,
    pub hash: String, // Hex encoded
    pub prev_block_hash: String, // Hex encoded
    pub nonce: u64,
    pub transactions: Vec<TransactionData>,
}

impl BlockData {
    pub fn from_block(block: &Block) -> Self {
        BlockData {
            index: block.index,
            timestamp: block.timestamp,
            hash: hex::encode(&block.hash),
            prev_block_hash: hex::encode(&block.prev_block_hash),
            nonce: block.nonce,
            transactions: block.transactions.iter()
                .map(TransactionData::from_transaction)
                .collect(),
        }
    }

    pub fn to_block(&self) -> Result<Block, String> {
        let hash = hex::decode(&self.hash)
            .map_err(|e| format!("Invalid hash hex: {}", e))?;
        let prev_block_hash = hex::decode(&self.prev_block_hash)
            .map_err(|e| format!("Invalid prev_block_hash hex: {}", e))?;

        let transactions: Result<Vec<_>, String> = self.transactions.iter()
            .map(|t| t.to_transaction())
            .collect();

        Ok(Block {
            index: self.index,
            timestamp: self.timestamp,
            hash,
            prev_block_hash,
            nonce: self.nonce,
            transactions: transactions?,
        })
    }
}

/// Serializable transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    pub inputs: Vec<OutputData>,
    pub outputs: Vec<OutputData>,
}

impl TransactionData {
    pub fn from_transaction(tx: &crate::transaction::Transaction) -> Self {
        TransactionData {
            inputs: tx.inputs.iter().map(OutputData::from_output).collect(),
            outputs: tx.outputs.iter().map(OutputData::from_output).collect(),
        }
    }

    pub fn to_transaction(&self) -> Result<crate::transaction::Transaction, String> {
        let inputs: Vec<_> = self.inputs.iter()
            .map(|o| o.to_output())
            .collect();
        let outputs: Vec<_> = self.outputs.iter()
            .map(|o| o.to_output())
            .collect();

        Ok(crate::transaction::Transaction {
            inputs,
            outputs,
        })
    }
}

/// Serializable output data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputData {
    pub to_addr: String,
    pub value: f64,
    pub timestamp: u128,
}

impl OutputData {
    pub fn from_output(output: &crate::transaction::Output) -> Self {
        OutputData {
            to_addr: output.to_addr.clone(),
            value: output.value,
            timestamp: output.timestamp,
        }
    }

    pub fn to_output(&self) -> crate::transaction::Output {
        crate::transaction::Output {
            to_addr: self.to_addr.clone(),
            value: self.value,
            timestamp: self.timestamp,
        }
    }
}

/// Round information data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundInfoData {
    pub round_start: u128,
    pub tonce: Option<u8>,
    pub challenge_seconds_remaining: u64,
    pub attempted_miners: usize,
    pub active_lockouts: usize,
    pub difficulty: String,
}

impl RoundInfoData {
    pub fn from_round_info(info: &RoundInfo, difficulty: u128) -> Self {
        RoundInfoData {
            round_start: info.round_start,
            tonce: info.tonce,
            challenge_seconds_remaining: info.challenge_seconds_remaining,
            attempted_miners: info.attempted_miners,
            active_lockouts: info.active_lockouts,
            difficulty: format!("0x{:X}", difficulty),
        }
    }
}

/// Block validation result types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockResultType {
    Accepted,
    RejectedInvalidHash,
    RejectedInvalidTimestamp,
    RejectedTonceChallenge,
    RejectedMinerInLockout,
    RejectedMinerAlreadyAttempted,
    RejectedBlockchainValidation,
}

impl From<&ValidationResult> for BlockResultType {
    fn from(result: &ValidationResult) -> Self {
        match result {
            ValidationResult::Accepted => BlockResultType::Accepted,
            ValidationResult::RejectedInvalidHash => BlockResultType::RejectedInvalidHash,
            ValidationResult::RejectedInvalidTimestamp => BlockResultType::RejectedInvalidTimestamp,
            ValidationResult::RejectedTonceChallenge => BlockResultType::RejectedTonceChallenge,
            ValidationResult::RejectedMinerInLockout => BlockResultType::RejectedMinerInLockout,
            ValidationResult::RejectedMinerAlreadyAttempted => BlockResultType::RejectedMinerAlreadyAttempted,
            ValidationResult::RejectedBlockchainValidation(_) => BlockResultType::RejectedBlockchainValidation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_data_serialization() {
        let block_data = BlockData {
            index: 0,
            timestamp: 1000,
            hash: "abcd".to_string(),
            prev_block_hash: "0000".to_string(),
            nonce: 123,
            transactions: vec![],
        };

        let json = serde_json::to_string(&block_data).unwrap();
        let deserialized: BlockData = serde_json::from_str(&json).unwrap();

        assert_eq!(block_data.index, deserialized.index);
        assert_eq!(block_data.timestamp, deserialized.timestamp);
    }

    #[test]
    fn test_miner_message_serialization() {
        let msg = MinerMessage::GetRoundInfo {
            miner_id: "test_miner".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: MinerMessage = serde_json::from_str(&json).unwrap();

        match deserialized {
            MinerMessage::GetRoundInfo { miner_id } => {
                assert_eq!(miner_id, "test_miner");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_validator_message_serialization() {
        let msg = ValidatorMessage::BlockResult {
            result: BlockResultType::Accepted,
            message: "Block accepted!".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: ValidatorMessage = serde_json::from_str(&json).unwrap();

        match deserialized {
            ValidatorMessage::BlockResult { result, message } => {
                assert!(matches!(result, BlockResultType::Accepted));
                assert_eq!(message, "Block accepted!");
            }
            _ => panic!("Wrong message type"),
        }
    }
}
