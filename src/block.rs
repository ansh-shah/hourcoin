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
	use crate::transaction::{Transaction, Output};

	#[test]
	fn test_block_creation() {
		let transactions = vec![Transaction {
			inputs: vec![],
			outputs: vec![Output {
				to_addr: "Alice".to_owned(),
				value: 2.0,
				timestamp: now(),
			}],
		}];

		let block = Block::new(0, now(), vec![0; 32], transactions);
		assert_eq!(block.index, 0);
		assert_eq!(block.nonce, 0);
		assert_eq!(block.prev_block_hash, vec![0; 32]);
		assert_eq!(block.transactions.len(), 1);
	}

	#[test]
	fn test_block_mining() {
		let difficulty = 0x00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
		let transactions = vec![Transaction {
			inputs: vec![],
			outputs: vec![Output {
				to_addr: "Alice".to_owned(),
				value: 2.0,
				timestamp: now(),
			}],
		}];

		let mut block = Block::new(0, now(), vec![0; 32], transactions);
		block.mine(difficulty);

		assert!(check_blockhash(&block.hash, difficulty));
		assert!(block.nonce > 0);
	}

	#[test]
	fn test_check_blockhash() {
		let easy_difficulty = 0x0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
		let hard_difficulty = 0x000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;

		// Hash with small value in last 16 bytes should pass easy difficulty
		// The difficulty_bytes_as_u128 function reads the last 16 bytes (indices 16-31)
		let easy_hash = vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
		assert!(check_blockhash(&easy_hash, easy_difficulty));

		// Hash with large value in last 16 bytes should fail hard difficulty
		let hard_hash = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
		assert!(!check_blockhash(&hard_hash, hard_difficulty));
	}

	#[test]
	fn test_block_hashing() {
		let transactions = vec![Transaction {
			inputs: vec![],
			outputs: vec![Output {
				to_addr: "Alice".to_owned(),
				value: 2.0,
				timestamp: 1000,
			}],
		}];

		let block1 = Block::new(0, 1000, vec![0; 32], transactions.clone());
		let hash1 = block1.hash();

		let block2 = Block::new(0, 1000, vec![0; 32], transactions.clone());
		let hash2 = block2.hash();

		// Same block data should produce same hash
		assert_eq!(hash1, hash2);

		// Different nonce should produce different hash
		let mut block3 = Block::new(0, 1000, vec![0; 32], transactions.clone());
		block3.nonce = 1;
		let hash3 = block3.hash();
		assert_ne!(hash1, hash3);
	}
}

