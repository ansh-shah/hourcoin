use super::*;
use std::collections::HashSet;

#[derive(Clone)]
pub struct Output {
	pub to_addr: Address,
	pub value: f64,
	pub timestamp: u128,
}

impl Hashable for Output {
	fn bytes (&self) -> Vec<u8> {
		let mut bytes = vec![];
		bytes.extend(self.to_addr.as_bytes());
		bytes.extend(&self.value.to_be_bytes());
		bytes.extend(&self.timestamp.to_be_bytes());

		bytes
	}
}

#[derive(Clone)]
pub struct Transaction {
	pub inputs: Vec<Output>,
	pub outputs: Vec<Output>,
}

impl Transaction {
	pub fn input_sum (&self) -> f64 {
		self.inputs.iter()
			.map(|input| input.value)
			.sum()
	}

	pub fn output_sum (&self) -> f64 {
		self.outputs.iter()
			.map(|output| output.value)
			.sum()
	}

	pub fn input_hashes (&self) -> HashSet<BlockHash> {
		self.inputs.iter()
			.map(|input| input.hash())
			.collect::<HashSet<BlockHash>>()
	}

	pub fn output_hashes (&self) -> HashSet<BlockHash> {
		self.outputs.iter()
			.map(|output| output.hash())
			.collect::<HashSet<BlockHash>>()
	}

	pub fn is_coinbase (&self) -> bool {
		(self.inputs.len() == 0) && (self.output_sum() == 2.0)
	}
}

impl Hashable for Transaction {
	fn bytes (&self) -> Vec<u8> {
		let mut bytes = vec![];

		bytes.extend(self.inputs.iter()
								.flat_map(|input| input.bytes())
								.collect::<Vec<u8>>());
		bytes.extend(self.outputs.iter()
								.flat_map(|output| output.bytes())
								.collect::<Vec<u8>>());

		bytes
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn create_output(addr: &str, value: f64, timestamp: u128) -> Output {
		Output {
			to_addr: addr.to_string(),
			value,
			timestamp,
		}
	}

	#[test]
	fn test_output_creation() {
		let output = create_output("Alice", 5.0, 1000);
		assert_eq!(output.to_addr, "Alice");
		assert_eq!(output.value, 5.0);
		assert_eq!(output.timestamp, 1000);
	}

	#[test]
	fn test_output_hash_consistency() {
		let output1 = create_output("Alice", 5.0, 1000);
		let output2 = create_output("Alice", 5.0, 1000);
		assert_eq!(output1.hash(), output2.hash());
	}

	#[test]
	fn test_output_hash_different_address() {
		let output1 = create_output("Alice", 5.0, 1000);
		let output2 = create_output("Bob", 5.0, 1000);
		assert_ne!(output1.hash(), output2.hash());
	}

	#[test]
	fn test_output_hash_different_value() {
		let output1 = create_output("Alice", 5.0, 1000);
		let output2 = create_output("Alice", 6.0, 1000);
		assert_ne!(output1.hash(), output2.hash());
	}

	#[test]
	fn test_output_hash_different_timestamp() {
		let output1 = create_output("Alice", 5.0, 1000);
		let output2 = create_output("Alice", 5.0, 2000);
		assert_ne!(output1.hash(), output2.hash());
	}

	#[test]
	fn test_transaction_input_sum() {
		let tx = Transaction {
			inputs: vec![
				create_output("Alice", 5.0, 1000),
				create_output("Bob", 3.0, 1000),
			],
			outputs: vec![],
		};
		assert_eq!(tx.input_sum(), 8.0);
	}

	#[test]
	fn test_transaction_output_sum() {
		let tx = Transaction {
			inputs: vec![],
			outputs: vec![
				create_output("Alice", 5.0, 1000),
				create_output("Bob", 3.0, 1000),
			],
		};
		assert_eq!(tx.output_sum(), 8.0);
	}

	#[test]
	fn test_transaction_empty_sums() {
		let tx = Transaction {
			inputs: vec![],
			outputs: vec![],
		};
		assert_eq!(tx.input_sum(), 0.0);
		assert_eq!(tx.output_sum(), 0.0);
	}

	#[test]
	fn test_transaction_fractional_values() {
		let tx = Transaction {
			inputs: vec![
				create_output("Alice", 1.5, 1000),
				create_output("Bob", 0.5, 1000),
			],
			outputs: vec![
				create_output("Charlie", 1.25, 1001),
				create_output("Dave", 0.75, 1001),
			],
		};
		assert_eq!(tx.input_sum(), 2.0);
		assert_eq!(tx.output_sum(), 2.0);
	}

	#[test]
	fn test_transaction_is_coinbase_true() {
		let tx = Transaction {
			inputs: vec![],
			outputs: vec![
				create_output("Alice", 1.5, 1000),
				create_output("Bob", 0.5, 1000),
			],
		};
		assert!(tx.is_coinbase());
	}

	#[test]
	fn test_transaction_is_coinbase_false_with_inputs() {
		let tx = Transaction {
			inputs: vec![create_output("Miner", 2.0, 999)],
			outputs: vec![
				create_output("Alice", 1.5, 1000),
				create_output("Bob", 0.5, 1000),
			],
		};
		assert!(!tx.is_coinbase());
	}

	#[test]
	fn test_transaction_is_coinbase_false_wrong_sum() {
		let tx = Transaction {
			inputs: vec![],
			outputs: vec![
				create_output("Alice", 1.0, 1000),
				create_output("Bob", 0.5, 1000),
			],
		};
		assert!(!tx.is_coinbase());
	}

	#[test]
	fn test_transaction_input_hashes_unique() {
		let tx = Transaction {
			inputs: vec![
				create_output("Alice", 5.0, 1000),
				create_output("Bob", 3.0, 1000),
			],
			outputs: vec![],
		};
		let hashes = tx.input_hashes();
		assert_eq!(hashes.len(), 2);
	}

	#[test]
	fn test_transaction_input_hashes_duplicate() {
		let output = create_output("Alice", 5.0, 1000);
		let tx = Transaction {
			inputs: vec![output.clone(), output.clone()],
			outputs: vec![],
		};
		let hashes = tx.input_hashes();
		// HashSet should deduplicate
		assert_eq!(hashes.len(), 1);
	}

	#[test]
	fn test_transaction_output_hashes_unique() {
		let tx = Transaction {
			inputs: vec![],
			outputs: vec![
				create_output("Alice", 5.0, 1000),
				create_output("Bob", 3.0, 1000),
			],
		};
		let hashes = tx.output_hashes();
		assert_eq!(hashes.len(), 2);
	}

	#[test]
	fn test_transaction_hash_consistency() {
		let tx1 = Transaction {
			inputs: vec![create_output("Alice", 5.0, 1000)],
			outputs: vec![create_output("Bob", 5.0, 1001)],
		};
		let tx2 = Transaction {
			inputs: vec![create_output("Alice", 5.0, 1000)],
			outputs: vec![create_output("Bob", 5.0, 1001)],
		};
		assert_eq!(tx1.hash(), tx2.hash());
	}

	#[test]
	fn test_transaction_hash_different_inputs() {
		let tx1 = Transaction {
			inputs: vec![create_output("Alice", 5.0, 1000)],
			outputs: vec![create_output("Bob", 5.0, 1001)],
		};
		let tx2 = Transaction {
			inputs: vec![create_output("Charlie", 5.0, 1000)],
			outputs: vec![create_output("Bob", 5.0, 1001)],
		};
		assert_ne!(tx1.hash(), tx2.hash());
	}

	#[test]
	fn test_output_bytes_includes_all_fields() {
		let output = create_output("Alice", 5.0, 1000);
		let bytes = output.bytes();

		// Should include address, value, and timestamp
		assert!(bytes.len() > 0);
		// Address "Alice" is 5 bytes + 8 bytes (f64) + 16 bytes (u128) = 29 bytes
		assert_eq!(bytes.len(), 29);
	}

	#[test]
	fn test_zero_value_output() {
		let output = create_output("Alice", 0.0, 1000);
		assert_eq!(output.value, 0.0);
		// Should still be hashable
		let hash = output.hash();
		assert_eq!(hash.len(), 32); // SHA256 produces 32 bytes
	}

	#[test]
	fn test_large_value_output() {
		let output = create_output("Alice", 1000000.0, 1000);
		assert_eq!(output.value, 1000000.0);
	}
}