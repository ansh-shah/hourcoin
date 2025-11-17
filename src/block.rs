use std::fmt::{self, Debug, Formatter};
use super::*;

#[derive(Clone)]
pub struct Block {
	pub index: u32, // block index
	pub timestamp: u128, // timestamp of when block is created
	pub hash: BlockHash, // current block hash
	pub prev_block_hash: BlockHash, //prev block hash
	pub nonce: u64, // for mining
	pub transactions: Vec<Transaction>, // will change for transactions



}

impl Debug for Block {
	fn fmt (&self, f: &mut Formatter) -> fmt::Result {
		// write!(f, "Block [{}]: {} at: {} with: {} nonce: {}", 
		// 	&self.index, &hex::encode(&self.hash), &self.timestamp, &self.transactions.len(), &self.nonce
		// )
		write!(f, "[Block #{} - hash: {}, timestamp: {}, nonce: {}]: transactions: {}",
				&self.index, &hex::encode(&self.hash), &self.timestamp, &self.nonce, &self.transactions.len())
	}
}

impl Block { 
	pub fn new(index: u32, timestamp: u128,  prev_block_hash: BlockHash, transactions: Vec<Transaction>,) -> Self {
		Block {
			index, 
			timestamp, 
			hash: vec![0; 32], 
			prev_block_hash, 
			nonce: 0, 
			transactions,
		}
	}

	pub fn mine (&mut self, difficulty: u128){
		for nonce_attempt in 0..(u64::max_value()){
			self.nonce = nonce_attempt;
			let hash = self.hash();
			if check_blockhash(&hash, difficulty){
				self.hash = hash;
				return;
			}
	}
}
}



impl Hashable for Block {
	fn bytes (&self) -> Vec<u8> {
		let mut bytes = vec![];

		bytes.extend(&u32_bytes(&self.index));
		bytes.extend(&u128_bytes(&self.timestamp));
		bytes.extend(&self.prev_block_hash);
		bytes.extend(&u64_bytes(&self.nonce));
		bytes.extend(self.transactions.iter()
									    .flat_map(|transaction| transaction.bytes())
									    .collect::<Vec<u8>>()
		);


		bytes
	}
}

pub fn check_blockhash (hash: &BlockHash, difficulty: u128) -> bool {
	difficulty > difficulty_bytes_as_u128(&hash)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::transaction::Output;

	fn create_output(addr: &str, value: f64, timestamp: u128) -> Output {
		Output {
			to_addr: addr.to_string(),
			value,
			timestamp,
		}
	}

	#[test]
	fn test_block_creation() {
		let transactions = vec![Transaction {
			inputs: vec![],
			outputs: vec![create_output("Alice", 1.5, 1000), create_output("Bob", 0.5, 1000)],
		}];
		let block = Block::new(0, 1000, vec![0; 32], transactions);

		assert_eq!(block.index, 0);
		assert_eq!(block.timestamp, 1000);
		assert_eq!(block.nonce, 0);
		assert_eq!(block.hash, vec![0; 32]);
		assert_eq!(block.prev_block_hash, vec![0; 32]);
		assert_eq!(block.transactions.len(), 1);
	}

	#[test]
	fn test_block_hash_consistency() {
		let transactions = vec![Transaction {
			inputs: vec![],
			outputs: vec![create_output("Alice", 1.5, 1000), create_output("Bob", 0.5, 1000)],
		}];
		let block1 = Block::new(0, 1000, vec![0; 32], transactions.clone());
		let block2 = Block::new(0, 1000, vec![0; 32], transactions);

		assert_eq!(block1.hash(), block2.hash());
	}

	#[test]
	fn test_block_hash_different_index() {
		let transactions = vec![Transaction {
			inputs: vec![],
			outputs: vec![create_output("Alice", 1.5, 1000), create_output("Bob", 0.5, 1000)],
		}];
		let block1 = Block::new(0, 1000, vec![0; 32], transactions.clone());
		let block2 = Block::new(1, 1000, vec![0; 32], transactions);

		assert_ne!(block1.hash(), block2.hash());
	}

	#[test]
	fn test_block_hash_different_timestamp() {
		let transactions = vec![Transaction {
			inputs: vec![],
			outputs: vec![create_output("Alice", 1.5, 1000), create_output("Bob", 0.5, 1000)],
		}];
		let block1 = Block::new(0, 1000, vec![0; 32], transactions.clone());
		let block2 = Block::new(0, 2000, vec![0; 32], transactions);

		assert_ne!(block1.hash(), block2.hash());
	}

	#[test]
	fn test_block_hash_different_prev_hash() {
		let transactions = vec![Transaction {
			inputs: vec![],
			outputs: vec![create_output("Alice", 1.5, 1000), create_output("Bob", 0.5, 1000)],
		}];
		let block1 = Block::new(0, 1000, vec![0; 32], transactions.clone());
		let block2 = Block::new(0, 1000, vec![1; 32], transactions);

		assert_ne!(block1.hash(), block2.hash());
	}

	#[test]
	fn test_block_hash_different_transactions() {
		let transactions1 = vec![Transaction {
			inputs: vec![],
			outputs: vec![create_output("Alice", 1.5, 1000), create_output("Bob", 0.5, 1000)],
		}];
		let transactions2 = vec![Transaction {
			inputs: vec![],
			outputs: vec![create_output("Charlie", 1.5, 1000), create_output("Dave", 0.5, 1000)],
		}];
		let block1 = Block::new(0, 1000, vec![0; 32], transactions1);
		let block2 = Block::new(0, 1000, vec![0; 32], transactions2);

		assert_ne!(block1.hash(), block2.hash());
	}

	#[test]
	fn test_check_blockhash_valid() {
		// Create a hash with low difficulty requirement (hash value should be less than difficulty)
		let mut hash = vec![0; 32];
		hash[31] = 0x0F; // Set last byte to low value
		let difficulty = u128::MAX;
		assert!(check_blockhash(&hash, difficulty));
	}

	#[test]
	fn test_check_blockhash_invalid() {
		// Create a hash with very high difficulty (low numeric value)
		let hash = vec![0xFF; 32];
		let difficulty = 0;
		assert!(!check_blockhash(&hash, difficulty));
	}

	#[test]
	fn test_check_blockhash_zero_hash() {
		// Hash of all zeros has the highest difficulty
		let hash = vec![0; 32];
		let difficulty = 1000;
		assert!(check_blockhash(&hash, difficulty));
	}

	#[test]
	fn test_block_mine_easy_difficulty() {
		let transactions = vec![Transaction {
			inputs: vec![],
			outputs: vec![create_output("Alice", 1.5, 1000), create_output("Bob", 0.5, 1000)],
		}];
		let mut block = Block::new(0, 1000, vec![0; 32], transactions);

		// Very easy difficulty - almost any hash will work
		let difficulty = u128::MAX;
		block.mine(difficulty);

		// After mining, the hash should be set and meet difficulty
		assert_ne!(block.hash, vec![0; 32]);
		assert!(check_blockhash(&block.hash, difficulty));
	}

	#[test]
	fn test_block_mine_moderate_difficulty() {
		let transactions = vec![Transaction {
			inputs: vec![],
			outputs: vec![create_output("Alice", 1.5, 1000), create_output("Bob", 0.5, 1000)],
		}];
		let mut block = Block::new(0, 1000, vec![0; 32], transactions);

		// Moderate difficulty
		let difficulty = 0x0000_ffff_ffff_ffff_ffff_ffff_ffff_ffff;
		block.mine(difficulty);

		// After mining, the hash should meet difficulty
		assert!(check_blockhash(&block.hash, difficulty));
		assert!(block.nonce > 0); // Should have tried multiple nonces
	}

	#[test]
	fn test_block_mine_updates_hash() {
		let transactions = vec![Transaction {
			inputs: vec![],
			outputs: vec![create_output("Alice", 1.5, 1000), create_output("Bob", 0.5, 1000)],
		}];
		let mut block = Block::new(0, 1000, vec![0; 32], transactions);

		let original_hash = block.hash.clone();
		let difficulty = u128::MAX;
		block.mine(difficulty);

		// Hash should be updated after mining
		assert_ne!(block.hash, original_hash);
	}

	#[test]
	fn test_block_bytes_includes_all_fields() {
		let transactions = vec![Transaction {
			inputs: vec![],
			outputs: vec![create_output("Alice", 1.5, 1000), create_output("Bob", 0.5, 1000)],
		}];
		let block = Block::new(0, 1000, vec![0; 32], transactions);

		let bytes = block.bytes();

		// Should include: index (4) + timestamp (16) + prev_hash (32) + nonce (8) + transactions
		// Minimum: 4 + 16 + 32 + 8 = 60 bytes, plus transaction data
		assert!(bytes.len() >= 60);
	}

	#[test]
	fn test_block_debug_format() {
		let transactions = vec![Transaction {
			inputs: vec![],
			outputs: vec![create_output("Alice", 1.5, 1000), create_output("Bob", 0.5, 1000)],
		}];
		let block = Block::new(0, 1000, vec![0; 32], transactions);

		let debug_str = format!("{:?}", block);
		assert!(debug_str.contains("Block #0"));
		assert!(debug_str.contains("timestamp: 1000"));
		assert!(debug_str.contains("nonce: 0"));
		assert!(debug_str.contains("transactions: 1"));
	}

	#[test]
	fn test_block_with_multiple_transactions() {
		let transactions = vec![
			Transaction {
				inputs: vec![],
				outputs: vec![create_output("Alice", 1.5, 1000), create_output("Bob", 0.5, 1000)],
			},
			Transaction {
				inputs: vec![create_output("Alice", 1.5, 1000)],
				outputs: vec![create_output("Charlie", 1.5, 1001)],
			},
		];
		let block = Block::new(0, 1000, vec![0; 32], transactions);

		assert_eq!(block.transactions.len(), 2);
	}

	#[test]
	fn test_block_with_no_transactions() {
		let block = Block::new(0, 1000, vec![0; 32], vec![]);

		assert_eq!(block.transactions.len(), 0);
		// Should still be hashable
		let hash = block.hash();
		assert_eq!(hash.len(), 32);
	}
}
