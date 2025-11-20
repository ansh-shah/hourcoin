/// Validator/Timekeeper Node for Hourcoin's Proof of Time consensus
///
/// The validator is responsible for:
/// 1. Maintaining the canonical blockchain
/// 2. Receiving block submissions from miners
/// 3. Validating timestamps against tonce challenges
/// 4. Enforcing the miner sacrifice protocol (1-hour lockout)
/// 5. Managing mining sessions and tracking miner attempts

use crate::{Block, Blockchain, now};
use crate::time_sync::TimeSync;
use crate::tonce::TonceChallenge;
use std::collections::{HashMap, HashSet};

/// Represents a miner's session with the validator
#[derive(Debug, Clone)]
pub struct MinerSession {
    pub miner_id: String,
    pub block_accepted_at: u128,
    pub must_wait_until: u128,
    pub is_active: bool,
}

impl MinerSession {
    /// Create a new miner session
    pub fn new(miner_id: String, block_accepted_at: u128) -> Self {
        let must_wait_until = block_accepted_at + 3_600_000; // 1 hour in milliseconds

        MinerSession {
            miner_id,
            block_accepted_at,
            must_wait_until,
            is_active: true,
        }
    }

    /// Check if the session lockout period has expired
    pub fn is_lockout_expired(&self, current_time: u128) -> bool {
        current_time >= self.must_wait_until
    }

    /// Get seconds remaining in lockout
    pub fn seconds_remaining(&self, current_time: u128) -> u64 {
        if self.is_lockout_expired(current_time) {
            0
        } else {
            ((self.must_wait_until - current_time) / 1000) as u64
        }
    }
}

/// Validation result for block submissions
#[derive(Debug, PartialEq)]
pub enum ValidationResult {
    Accepted,
    RejectedInvalidHash,
    RejectedInvalidTimestamp,
    RejectedTonceChallenge,
    RejectedMinerInLockout,
    RejectedMinerAlreadyAttempted,
    RejectedBlockchainValidation(String),
}

/// The Validator node that manages the proof of time consensus
pub struct Validator {
    /// The canonical blockchain maintained by the validator
    pub blockchain: Blockchain,
    /// Time synchronization service
    time_sync: TimeSync,
    /// Current tonce challenge
    current_tonce: Option<TonceChallenge>,
    /// Active miner sessions (miners in 1-hour lockout)
    active_sessions: HashMap<String, MinerSession>,
    /// Miners who have attempted in the current mining round
    attempted_this_round: HashSet<String>,
    /// The timestamp when the current mining round started
    current_round_start: u128,
}

impl Validator {
    /// Create a new validator with a specified blockchain difficulty
    pub fn new(difficulty: u128) -> Self {
        Validator {
            blockchain: Blockchain::new_with_diff(difficulty),
            time_sync: TimeSync::new(),
            current_tonce: None,
            active_sessions: HashMap::new(),
            attempted_this_round: HashSet::new(),
            current_round_start: now(),
        }
    }

    /// Initialize the tonce challenge for a new mining round
    pub fn start_new_round(&mut self) {
        let prev_timestamp = if let Some(last_block) = self.blockchain.blocks.last() {
            last_block.timestamp
        } else {
            now()
        };

        self.current_tonce = Some(TonceChallenge::new(prev_timestamp));
        self.current_round_start = now();
        self.attempted_this_round.clear();

        // Clean up expired sessions
        let current_time = now();
        self.active_sessions.retain(|_, session| {
            !session.is_lockout_expired(current_time)
        });
    }

    /// Validate and potentially accept a block submission from a miner
    pub fn validate_block_submission(
        &mut self,
        block: Block,
        miner_id: String,
    ) -> ValidationResult {
        let current_time = now();

        // Check if miner is in lockout period (miner sacrifice protocol)
        if let Some(session) = self.active_sessions.get(&miner_id) {
            if !session.is_lockout_expired(current_time) {
                return ValidationResult::RejectedMinerInLockout;
            }
        }

        // Check if miner has already attempted this round (prevent spam)
        if self.attempted_this_round.contains(&miner_id) {
            return ValidationResult::RejectedMinerAlreadyAttempted;
        }

        // Mark that this miner has attempted this round
        self.attempted_this_round.insert(miner_id.clone());

        // Validate timestamp against time sync
        if !self.time_sync.validate_timestamp(block.timestamp) {
            return ValidationResult::RejectedInvalidTimestamp;
        }

        // Validate against tonce challenge
        if let Some(ref mut tonce) = self.current_tonce {
            if !tonce.validate_timestamp(block.timestamp, current_time) {
                return ValidationResult::RejectedTonceChallenge;
            }
        }

        // Validate against blockchain rules
        match self.blockchain.update_with_block(block.clone()) {
            Ok(_) => {
                // Block accepted! Start miner sacrifice period
                let session = MinerSession::new(miner_id.clone(), current_time);
                self.active_sessions.insert(miner_id, session);

                // Start new mining round
                self.start_new_round();

                ValidationResult::Accepted
            }
            Err(e) => ValidationResult::RejectedBlockchainValidation(format!("{:?}", e)),
        }
    }

    /// Get the current tonce value
    pub fn get_current_tonce(&self) -> Option<u8> {
        self.current_tonce.as_ref().map(|t| t.get_tonce())
    }

    /// Get time remaining in current tonce challenge (seconds)
    pub fn get_challenge_time_remaining(&self) -> u64 {
        if let Some(ref tonce) = self.current_tonce {
            tonce.seconds_remaining(now())
        } else {
            0
        }
    }

    /// Check if a miner is currently in lockout
    pub fn is_miner_in_lockout(&self, miner_id: &str) -> bool {
        if let Some(session) = self.active_sessions.get(miner_id) {
            !session.is_lockout_expired(now())
        } else {
            false
        }
    }

    /// Get lockout time remaining for a miner (seconds)
    pub fn get_miner_lockout_remaining(&self, miner_id: &str) -> u64 {
        if let Some(session) = self.active_sessions.get(miner_id) {
            session.seconds_remaining(now())
        } else {
            0
        }
    }

    /// Get the number of blocks in the blockchain
    pub fn get_block_count(&self) -> usize {
        self.blockchain.blocks.len()
    }

    /// Get the current difficulty
    pub fn get_difficulty(&self) -> u128 {
        self.blockchain.get_difficulty()
    }

    /// Get information about the current mining round
    pub fn get_round_info(&self) -> RoundInfo {
        RoundInfo {
            round_start: self.current_round_start,
            tonce: self.get_current_tonce(),
            challenge_seconds_remaining: self.get_challenge_time_remaining(),
            attempted_miners: self.attempted_this_round.len(),
            active_lockouts: self.active_sessions.len(),
        }
    }
}

/// Information about the current mining round
#[derive(Debug, Clone)]
pub struct RoundInfo {
    pub round_start: u128,
    pub tonce: Option<u8>,
    pub challenge_seconds_remaining: u64,
    pub attempted_miners: usize,
    pub active_lockouts: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{Transaction, Output};

    fn create_test_block(index: u32, timestamp: u128, prev_hash: Vec<u8>, difficulty: u128) -> Block {
        let coinbase = Transaction {
            inputs: vec![],
            outputs: vec![Output {
                to_addr: "Miner".to_owned(),
                value: 2.0,
                timestamp,
            }],
        };

        let mut block = Block::new(index, timestamp, prev_hash, vec![coinbase]);
        block.mine(difficulty);
        block
    }

    #[test]
    fn test_validator_creation() {
        let difficulty = 0x00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
        let validator = Validator::new(difficulty);

        assert_eq!(validator.get_block_count(), 0);
        assert_eq!(validator.get_difficulty(), difficulty);
    }

    #[test]
    fn test_miner_session_creation() {
        let session = MinerSession::new("miner1".to_string(), 1000000);

        assert_eq!(session.miner_id, "miner1");
        assert_eq!(session.block_accepted_at, 1000000);
        assert_eq!(session.must_wait_until, 1000000 + 3_600_000);
        assert!(session.is_active);
    }

    #[test]
    fn test_miner_session_lockout() {
        let session = MinerSession::new("miner1".to_string(), 1000000);

        // During lockout
        assert!(!session.is_lockout_expired(1000000 + 1000));
        assert!(session.seconds_remaining(1000000 + 1000) > 0);

        // After lockout
        assert!(session.is_lockout_expired(1000000 + 3_700_000));
        assert_eq!(session.seconds_remaining(1000000 + 3_700_000), 0);
    }

    #[test]
    fn test_start_new_round() {
        let difficulty = 0x00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
        let mut validator = Validator::new(difficulty);

        validator.start_new_round();

        assert!(validator.current_tonce.is_some());
        assert!(validator.get_current_tonce().is_some());
        assert_eq!(validator.attempted_this_round.len(), 0);
    }

    #[test]
    fn test_get_round_info() {
        let difficulty = 0x00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
        let mut validator = Validator::new(difficulty);

        validator.start_new_round();
        let info = validator.get_round_info();

        assert!(info.tonce.is_some());
        assert_eq!(info.attempted_miners, 0);
        assert_eq!(info.active_lockouts, 0);
    }

    #[test]
    fn test_miner_lockout_tracking() {
        let difficulty = 0x00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
        let mut validator = Validator::new(difficulty);

        // Miner not in lockout initially
        assert!(!validator.is_miner_in_lockout("miner1"));

        // Add a session
        let session = MinerSession::new("miner1".to_string(), now());
        validator.active_sessions.insert("miner1".to_string(), session);

        // Miner should now be in lockout
        assert!(validator.is_miner_in_lockout("miner1"));
        assert!(validator.get_miner_lockout_remaining("miner1") > 0);
    }

    #[test]
    fn test_validation_result_equality() {
        assert_eq!(ValidationResult::Accepted, ValidationResult::Accepted);
        assert_eq!(
            ValidationResult::RejectedInvalidHash,
            ValidationResult::RejectedInvalidHash
        );
        assert_ne!(ValidationResult::Accepted, ValidationResult::RejectedInvalidHash);
    }
}
