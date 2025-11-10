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

	#[test]
	fn test_output_creation() {
		let output = Output {
			to_addr: "Alice".to_owned(),
			value: 10.0,
			timestamp: 1000,
		};
		assert_eq!(output.to_addr, "Alice");
		assert_eq!(output.value, 10.0);
		assert_eq!(output.timestamp, 1000);
	}

	#[test]
	fn test_output_hashing() {
		let output1 = Output {
			to_addr: "Alice".to_owned(),
			value: 10.0,
			timestamp: 1000,
		};
		let output2 = Output {
			to_addr: "Alice".to_owned(),
			value: 10.0,
			timestamp: 1000,
		};
		let output3 = Output {
			to_addr: "Bob".to_owned(),
			value: 10.0,
			timestamp: 1000,
		};

		// Same outputs should produce same hash
		assert_eq!(output1.hash(), output2.hash());

		// Different outputs should produce different hash
		assert_ne!(output1.hash(), output3.hash());
	}

	#[test]
	fn test_coinbase_transaction() {
		let coinbase = Transaction {
			inputs: vec![],
			outputs: vec![Output {
				to_addr: "Miner".to_owned(),
				value: 2.0,
				timestamp: 1000,
			}],
		};

		assert!(coinbase.is_coinbase());
		assert_eq!(coinbase.input_sum(), 0.0);
		assert_eq!(coinbase.output_sum(), 2.0);
	}

	#[test]
	fn test_non_coinbase_transaction() {
		let transaction = Transaction {
			inputs: vec![Output {
				to_addr: "Alice".to_owned(),
				value: 10.0,
				timestamp: 1000,
			}],
			outputs: vec![
				Output {
					to_addr: "Bob".to_owned(),
					value: 7.0,
					timestamp: 2000,
				},
				Output {
					to_addr: "Alice".to_owned(),
					value: 2.5,
					timestamp: 2000,
				},
			],
		};

		assert!(!transaction.is_coinbase());
		assert_eq!(transaction.input_sum(), 10.0);
		assert_eq!(transaction.output_sum(), 9.5);
	}

	#[test]
	fn test_transaction_input_hashes() {
		let input1 = Output {
			to_addr: "Alice".to_owned(),
			value: 10.0,
			timestamp: 1000,
		};
		let input2 = Output {
			to_addr: "Bob".to_owned(),
			value: 5.0,
			timestamp: 1000,
		};

		let transaction = Transaction {
			inputs: vec![input1.clone(), input2.clone()],
			outputs: vec![],
		};

		let input_hashes = transaction.input_hashes();
		assert_eq!(input_hashes.len(), 2);
		assert!(input_hashes.contains(&input1.hash()));
		assert!(input_hashes.contains(&input2.hash()));
	}

	#[test]
	fn test_transaction_output_hashes() {
		let output1 = Output {
			to_addr: "Alice".to_owned(),
			value: 10.0,
			timestamp: 2000,
		};
		let output2 = Output {
			to_addr: "Bob".to_owned(),
			value: 5.0,
			timestamp: 2000,
		};

		let transaction = Transaction {
			inputs: vec![],
			outputs: vec![output1.clone(), output2.clone()],
		};

		let output_hashes = transaction.output_hashes();
		assert_eq!(output_hashes.len(), 2);
		assert!(output_hashes.contains(&output1.hash()));
		assert!(output_hashes.contains(&output2.hash()));
	}

	#[test]
	fn test_invalid_coinbase_wrong_value() {
		let transaction = Transaction {
			inputs: vec![],
			outputs: vec![Output {
				to_addr: "Miner".to_owned(),
				value: 5.0, // Wrong value - should be 2.0
				timestamp: 1000,
			}],
		};

		assert!(!transaction.is_coinbase());
	}

	#[test]
	fn test_invalid_coinbase_has_inputs() {
		let transaction = Transaction {
			inputs: vec![Output {
				to_addr: "Someone".to_owned(),
				value: 2.0,
				timestamp: 1000,
			}],
			outputs: vec![Output {
				to_addr: "Miner".to_owned(),
				value: 2.0,
				timestamp: 1000,
			}],
		};

		assert!(!transaction.is_coinbase());
	}

	#[test]
	fn test_transaction_with_fractional_values() {
		let transaction = Transaction {
			inputs: vec![Output {
				to_addr: "Alice".to_owned(),
				value: 10.5,
				timestamp: 1000,
			}],
			outputs: vec![
				Output {
					to_addr: "Bob".to_owned(),
					value: 7.25,
					timestamp: 2000,
				},
				Output {
					to_addr: "Charlie".to_owned(),
					value: 3.0,
					timestamp: 2000,
				},
			],
		};

		assert_eq!(transaction.input_sum(), 10.5);
		assert_eq!(transaction.output_sum(), 10.25);
	}
}