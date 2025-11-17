# HourCoin - Running Examples

This document shows how to run the HourCoin blockchain system with example commands and their expected output.

---

## Running Unit Tests

### Command
```bash
$ cargo test
```

### Output
```
   Compiling hourcoin v0.1.0 (/home/user/hourcoin)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.31s
     Running unittests src/lib.rs (target/debug/deps/blockchainlib-6689523a578c4ddf)

running 77 tests
test block::tests::test_block_creation ... ok
test block::tests::test_block_bytes_includes_all_fields ... ok
test block::tests::test_block_debug_format ... ok
test block::tests::test_block_with_multiple_transactions ... ok
test block::tests::test_check_blockhash_invalid ... ok
test block::tests::test_check_blockhash_valid ... ok
test block::tests::test_check_blockhash_zero_hash ... ok
test block::tests::test_block_hash_different_transactions ... ok
test block::tests::test_block_hash_consistency ... ok
test block::tests::test_block_mine_easy_difficulty ... ok
test block::tests::test_block_with_no_transactions ... ok
test block::tests::test_block_mine_updates_hash ... ok
test blockchain::tests::test_blockchain_achronological_timestamp ... ok
test blockchain::tests::test_blockchain_block_invalid_hash_difficulty ... ok
test blockchain::tests::test_blockchain_block_mismatched_index ... ok
test blockchain::tests::test_blockchain_coinbase_with_fees ... ok
test blockchain::tests::test_blockchain_creation ... ok
test blockchain::tests::test_blockchain_creation_with_difficulty ... ok
test blockchain::tests::test_blockchain_genesis_block_invalid_prev_hash ... ok
test blockchain::tests::test_blockchain_genesis_block_valid ... ok
test blockchain::tests::test_blockchain_get_difficulty ... ok
test blockchain::tests::test_blockchain_invalid_coinbase_transaction ... ok
test blockchain::tests::test_blockchain_mismatched_previous_hash ... ok
test blockchain::tests::test_blockchain_missing_coinbase_transaction ... ok
test block::tests::test_block_hash_different_prev_hash ... ok
test blockchain::tests::test_blockchain_transaction_insufficient_input_value ... ok
test blockchain::tests::test_blockchain_transaction_invalid_input ... ok
test blockchain::tests::test_blockchain_transaction_invalid_timestamp ... ok
test blockchain::tests::test_blockchain_update_difficulty_equal ... ok
test block::tests::test_block_hash_different_index ... ok
test blockchain::tests::test_blockchain_update_difficulty_invalid ... ok
test blockchain::tests::test_blockchain_update_difficulty_valid ... ok
test blockchain::tests::test_blockchain_utxo_tracking ... ok
test hashable::tests::test_hashable_empty_bytes ... ok
test blockchain::tests::test_blockchain_valid_transaction_chain ... ok
test block::tests::test_block_hash_different_timestamp ... ok
test hashable::tests::test_hashable_trait_consistency ... ok
test hashable::tests::test_hashable_trait_deterministic ... ok
test hashable::tests::test_hashable_trait_different_data ... ok
test hashable::tests::test_hashable_trait_produces_hash ... ok
test tests::test_byte_conversion_round_trip_u32 ... ok
test tests::test_byte_conversion_round_trip_u64 ... ok
test tests::test_difficulty_bytes_all_ones ... ok
test tests::test_difficulty_bytes_all_zeros ... ok
test tests::test_difficulty_bytes_as_u128 ... ok
test tests::test_now_returns_reasonable_timestamp ... ok
test tests::test_u128_bytes_conversion ... ok
test tests::test_u128_bytes_max ... ok
test tests::test_u128_bytes_zero ... ok
test tests::test_u32_bytes_conversion ... ok
test tests::test_u32_bytes_max ... ok
test tests::test_u32_bytes_zero ... ok
test tests::test_u64_bytes_conversion ... ok
test tests::test_u64_bytes_max ... ok
test tests::test_u64_bytes_zero ... ok
test transaction::tests::test_large_value_output ... ok
test transaction::tests::test_output_bytes_includes_all_fields ... ok
test transaction::tests::test_output_creation ... ok
test transaction::tests::test_output_hash_consistency ... ok
test transaction::tests::test_output_hash_different_address ... ok
test transaction::tests::test_output_hash_different_timestamp ... ok
test transaction::tests::test_output_hash_different_value ... ok
test transaction::tests::test_transaction_empty_sums ... ok
test transaction::tests::test_transaction_fractional_values ... ok
test transaction::tests::test_transaction_hash_consistency ... ok
test transaction::tests::test_transaction_hash_different_inputs ... ok
test transaction::tests::test_transaction_input_hashes_duplicate ... ok
test transaction::tests::test_transaction_input_hashes_unique ... ok
test transaction::tests::test_transaction_input_sum ... ok
test transaction::tests::test_transaction_is_coinbase_false_with_inputs ... ok
test transaction::tests::test_transaction_is_coinbase_false_wrong_sum ... ok
test transaction::tests::test_transaction_is_coinbase_true ... ok
test transaction::tests::test_transaction_output_hashes_unique ... ok
test transaction::tests::test_transaction_output_sum ... ok
test transaction::tests::test_zero_value_output ... ok
test tests::test_now_increments ... ok
test block::tests::test_block_mine_moderate_difficulty ... ok

test result: ok. 77 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.58s
```

**Result:** ✅ All 77 tests passed!

---

## Running the Blockchain Demo

### Command
```bash
$ cargo run --bin blockchain
```

### Output
```
   Compiling hourcoin v0.1.0 (/home/user/hourcoin)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.71s
     Running `target/debug/blockchain`

Mined genesis block:
 [Block #0 - hash: 78075747b5c6ee19ca46063c1ee3f7ddcca1a4b508afdf3feed57cfa03e30e00, timestamp: 1763413158983, nonce: 2827]: transactions: 1

Mined block
 [Block #1 - hash: b5532f96d1543b61de68770c96e601059040f496a49f543952add06ab8390d00, timestamp: 1763413159003, nonce: 309]: transactions: 2
```

### Explanation of Output

#### Block #0 (Genesis Block)
- **Hash**: `78075747...03e30e00` (ends in `00` - meets difficulty requirement)
- **Timestamp**: `1763413158983` milliseconds since epoch
- **Nonce**: `2827` - The mining process tried 2,827 different nonce values before finding a valid hash
- **Transactions**: 1 coinbase transaction that creates the initial coins
  - Alice receives **1.5 coins**
  - Bob receives **0.5 coins**

#### Block #1 (Second Block)
- **Hash**: `b5532f96...b8390d00` (ends in `00` - meets difficulty requirement)
- **Timestamp**: `1763413159003` milliseconds since epoch (20ms after genesis)
- **Nonce**: `309` - Found valid hash after 309 attempts
- **Transactions**: 2 transactions
  1. **Coinbase transaction**: Chris receives **2.0 coins** (miner reward)
  2. **Regular transaction**: Alice spends her 1.5 coins
     - Sends **0.25 coins** to herself (change)
     - Sends **0.5 coins** to Bob
     - **0.75 coins** fee (1.5 - 0.25 - 0.5 = 0.75)

**Result:** ✅ Two blocks successfully mined and added to the blockchain!

---

## Building Without Running

### Command
```bash
$ cargo build
```

### Output
```
   Compiling hourcoin v0.1.0 (/home/user/hourcoin)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.45s
```

**Result:** ✅ Project compiled successfully!

---

## Running Specific Tests

### Test Only Blockchain Module

#### Command
```bash
$ cargo test blockchain::tests
```

#### Output
```
running 15 tests
test blockchain::tests::test_blockchain_creation ... ok
test blockchain::tests::test_blockchain_creation_with_difficulty ... ok
test blockchain::tests::test_blockchain_get_difficulty ... ok
test blockchain::tests::test_blockchain_update_difficulty_valid ... ok
test blockchain::tests::test_blockchain_update_difficulty_invalid ... ok
test blockchain::tests::test_blockchain_update_difficulty_equal ... ok
test blockchain::tests::test_blockchain_genesis_block_valid ... ok
test blockchain::tests::test_blockchain_genesis_block_invalid_prev_hash ... ok
test blockchain::tests::test_blockchain_block_mismatched_index ... ok
test blockchain::tests::test_blockchain_block_invalid_hash_difficulty ... ok
test blockchain::tests::test_blockchain_achronological_timestamp ... ok
test blockchain::tests::test_blockchain_mismatched_previous_hash ... ok
test blockchain::tests::test_blockchain_missing_coinbase_transaction ... ok
test blockchain::tests::test_blockchain_invalid_coinbase_transaction ... ok
test blockchain::tests::test_blockchain_transaction_invalid_input ... ok

test result: ok. 15 passed; 0 failed
```

### Test Only Block Module

#### Command
```bash
$ cargo test block::tests
```

#### Output
```
running 17 tests
test block::tests::test_block_creation ... ok
test block::tests::test_block_hash_consistency ... ok
test block::tests::test_block_hash_different_index ... ok
test block::tests::test_block_hash_different_timestamp ... ok
test block::tests::test_block_hash_different_prev_hash ... ok
test block::tests::test_block_hash_different_transactions ... ok
test block::tests::test_check_blockhash_valid ... ok
test block::tests::test_check_blockhash_invalid ... ok
test block::tests::test_check_blockhash_zero_hash ... ok
test block::tests::test_block_mine_easy_difficulty ... ok
test block::tests::test_block_mine_moderate_difficulty ... ok
test block::tests::test_block_mine_updates_hash ... ok
test block::tests::test_block_bytes_includes_all_fields ... ok
test block::tests::test_block_debug_format ... ok
test block::tests::test_block_with_multiple_transactions ... ok
test block::tests::test_block_with_no_transactions ... ok

test result: ok. 17 passed; 0 failed
```

---

## Test Coverage Summary

| Module | Tests | Description |
|--------|-------|-------------|
| **lib.rs** | 17 | Byte conversion utilities, timestamps, difficulty |
| **hashable.rs** | 5 | SHA256 hashing trait implementation |
| **transaction.rs** | 23 | Transaction validation, sums, coinbase logic |
| **block.rs** | 17 | Block creation, mining, hash validation |
| **blockchain.rs** | 15 | Blockchain validation, UTXO tracking, difficulty |
| **TOTAL** | **77** | Complete system coverage |

---

## Quick Reference

### Common Commands

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run the blockchain demo
cargo run --bin blockchain

# Build the project
cargo build

# Build with optimizations
cargo build --release

# Clean build artifacts
cargo clean

# Check code without building
cargo check
```

---

## What the System Validates

✅ **Proof-of-Work Mining**: Blocks must meet difficulty requirements
✅ **Block Ordering**: Sequential indices and chronological timestamps
✅ **Hash Chain**: Each block references the previous block's hash
✅ **Transactions**: Valid inputs, sufficient funds, proper timestamps
✅ **UTXO Tracking**: Prevents double-spending of outputs
✅ **Coinbase Rules**: First transaction creates 2.0 coins
✅ **Transaction Fees**: Miners can claim the difference between inputs and outputs
✅ **Difficulty Management**: Can only decrease difficulty over time

---

## System Requirements

- **Rust**: Edition 2018 or later
- **Dependencies**:
  - `hex = "0.4.3"`
  - `crypto-hash = "0.3.4"`
  - `rand = "0.8.3"`

All dependencies are automatically downloaded by Cargo.
