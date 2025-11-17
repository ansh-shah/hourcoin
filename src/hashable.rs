use super::*;
pub trait Hashable {
	fn bytes (&self) -> Vec<u8>;

	fn hash (&self) -> BlockHash {
		crypto_hash::digest(crypto_hash::Algorithm::SHA256, &self.bytes())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	// Create a simple test struct to verify the trait implementation
	struct TestData {
		value: u32,
		name: String,
	}

	impl Hashable for TestData {
		fn bytes(&self) -> Vec<u8> {
			let mut bytes = vec![];
			bytes.extend(&u32_bytes(&self.value));
			bytes.extend(self.name.as_bytes());
			bytes
		}
	}

	#[test]
	fn test_hashable_trait_produces_hash() {
		let data = TestData {
			value: 42,
			name: "test".to_string(),
		};
		let hash = data.hash();

		// SHA256 always produces 32 bytes
		assert_eq!(hash.len(), 32);
	}

	#[test]
	fn test_hashable_trait_consistency() {
		let data1 = TestData {
			value: 42,
			name: "test".to_string(),
		};
		let data2 = TestData {
			value: 42,
			name: "test".to_string(),
		};

		// Same data should produce same hash
		assert_eq!(data1.hash(), data2.hash());
	}

	#[test]
	fn test_hashable_trait_different_data() {
		let data1 = TestData {
			value: 42,
			name: "test".to_string(),
		};
		let data2 = TestData {
			value: 43,
			name: "test".to_string(),
		};

		// Different data should produce different hash
		assert_ne!(data1.hash(), data2.hash());
	}

	#[test]
	fn test_hashable_trait_deterministic() {
		let data = TestData {
			value: 42,
			name: "test".to_string(),
		};

		let hash1 = data.hash();
		let hash2 = data.hash();

		// Multiple calls should produce the same hash
		assert_eq!(hash1, hash2);
	}

	#[test]
	fn test_hashable_empty_bytes() {
		struct EmptyData;

		impl Hashable for EmptyData {
			fn bytes(&self) -> Vec<u8> {
				vec![]
			}
		}

		let data = EmptyData;
		let hash = data.hash();

		// Even empty data should produce a valid hash
		assert_eq!(hash.len(), 32);
	}
}
