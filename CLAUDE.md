# CLAUDE.md - Hourcoin AI Assistant Guide

## Project Overview

Hourcoin is an experimental Rust-based cryptocurrency implementing a novel **Proof of Time** consensus mechanism. Unlike traditional Proof-of-Work systems that value computational power, Hourcoin values miners' time - each coin represents one hour of a miner's life.

## Quick Reference

```bash
# Build the project
cargo build --release

# Run tests (60 tests should pass)
cargo test

# Run main demo
cargo run --release

# Run Proof of Time demo
cargo run --release --example proof_of_time_demo

# Start validator server
./target/release/validator [address] [difficulty]
# Default: 127.0.0.1:8080

# Start miner client
./target/release/miner <miner_id> [validator_address] [reward_address]
```

## Codebase Architecture

```
hourcoin/
├── Cargo.toml              # Package manifest (edition 2018)
├── src/
│   ├── lib.rs              # Library exports, type aliases, utility functions
│   ├── main.rs             # Main demo program
│   ├── block.rs            # Block structure, PoW mining, hash validation
│   ├── blockchain.rs       # UTXO-based blockchain, validation rules
│   ├── transaction.rs      # Transaction/Output structures, coinbase logic
│   ├── hashable.rs         # SHA-256 hashing trait
│   ├── time_sync.rs        # Time synchronization with external sources
│   ├── tonce.rs            # Tonce (time-only-used-once) challenge system
│   ├── leap_seconds.rs     # TAI time handling for leap second safety
│   ├── validator.rs        # Validator/timekeeper node logic
│   ├── network/
│   │   ├── mod.rs          # Network module exports
│   │   ├── protocol.rs     # JSON message protocol definitions
│   │   ├── validator_server.rs  # TCP server for validators
│   │   └── miner_client.rs      # TCP client for miners
│   └── bin/
│       ├── validator.rs    # Standalone validator binary
│       └── miner.rs        # Standalone miner binary
├── examples/
│   └── proof_of_time_demo.rs  # Complete PoT demonstration
└── docs/
    ├── Roadmap.md          # Project development plan
    ├── ProofOfTime.md      # Technical consensus documentation
    ├── NetworkGuide.md     # Distributed mining setup guide
    └── LeapSeconds.md      # TAI implementation details
```

## Key Concepts

### 1. Proof of Time Consensus

The consensus mechanism combines:

- **Tonce Challenges**: Time-based mining puzzles where miners find timestamps whose hash is divisible by a random value (1-31)
- **Miner Sacrifice Protocol**: Miners who successfully mine a block enter a 1-hour lockout period
- **Fair Mining**: Time-based challenges level the playing field regardless of hardware

### 2. Core Components

| Module | Purpose |
|--------|---------|
| `Block` | Holds index, timestamp, hash, prev_hash, nonce, transactions |
| `Blockchain` | Manages UTXO set, validates blocks and transactions |
| `Transaction` | Contains inputs/outputs, validates coinbase (2.0 hourcoin reward) |
| `TonceChallenge` | Calculates and validates time-based mining challenges |
| `Validator` | Manages consensus, enforces lockouts, tracks mining rounds |
| `TimeSync` | Validates timestamps, handles external time sync |
| `leap_seconds` | Converts UTC to TAI for monotonic, leap-second-safe timing |

### 3. Type Aliases

```rust
type BlockHash = Vec<u8>;  // SHA-256 hash (32 bytes)
type Address = String;      // Simple string addresses
```

## Development Guidelines

### Coding Conventions

- **Rust Edition**: 2018
- **Time Handling**: Always use TAI (via `now()` function) for consensus-critical timing, not UTC
- **Hashing**: Use `Hashable` trait which implements SHA-256 via `crypto_hash`
- **Serialization**: Use serde for JSON serialization in network protocol
- **Async Runtime**: Tokio for async networking operations

### Important Constants

```rust
// Tonce challenge duration
const TONCE_CHALLENGE_DURATION_MS: u128 = 60_000; // 60 seconds

// Miner lockout period
const LOCKOUT_DURATION_MS: u128 = 3_600_000; // 1 hour

// Coinbase reward
const COINBASE_VALUE: f64 = 2.0; // 1 for work + 1 for time sacrifice

// Default difficulty
const DEFAULT_DIFFICULTY: u128 = 0x00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;

// Time validation tolerance
const DEFAULT_TOLERANCE_MS: u128 = 500;
```

### Testing

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test block::tests
cargo test blockchain::tests
cargo test tonce::tests
cargo test validator::tests
cargo test time_sync::tests
cargo test leap_seconds::tests
cargo test network::protocol::tests
```

### Adding New Features

1. **Time-sensitive code**: Use `crate::now()` which returns TAI milliseconds
2. **Block validation**: Add rules in `blockchain.rs::update_with_block()`
3. **Network messages**: Add variants to `MinerMessage`/`ValidatorMessage` enums
4. **Consensus changes**: Modify `validator.rs::validate_block_submission()`

## Common Tasks

### Creating a Valid Block

```rust
use blockchainlib::*;

// Find timestamp satisfying tonce challenge
let tonce = validator.get_current_tonce().unwrap();
let valid_timestamp = find_valid_timestamp(tonce, now(), 10000)?;

// Create coinbase transaction
let coinbase = Transaction {
    inputs: vec![],
    outputs: vec![transaction::Output {
        to_addr: "miner_address".to_owned(),
        value: 2.0,  // Must be exactly 2.0 for coinbase
        timestamp: valid_timestamp,
    }],
};

// Create and mine block
let mut block = Block::new(index, valid_timestamp, prev_hash, vec![coinbase]);
block.mine(difficulty);
```

### Validating Timestamps

```rust
// For consensus: use TAI time
let current_tai = now(); // TAI milliseconds

// Check if timestamp is valid
let time_sync = TimeSync::new();
let is_valid = time_sync.validate_timestamp(block.timestamp);

// Check tonce challenge
let mut tonce = TonceChallenge::new(prev_block_timestamp);
let passes = tonce.validate_timestamp(candidate_timestamp, current_tai);
```

## Validation Rules

### Block Validation (in order)
1. Index matches expected (sequential)
2. Hash meets difficulty requirement
3. Timestamp is after previous block's timestamp
4. Previous hash matches last block's hash
5. Genesis block has prev_hash of all zeros
6. First transaction is valid coinbase (no inputs, outputs sum to 2.0)
7. All other transactions have valid inputs from UTXO set
8. Output timestamps >= input timestamps

### Proof of Time Validation
1. Miner not in lockout period
2. Miner hasn't already attempted this round
3. Timestamp within tolerance of current time
4. Timestamp passes tonce challenge (first 60 seconds)

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `hex` | 0.4.3 | Hex encoding/decoding |
| `crypto-hash` | 0.3.4 | SHA-256 hashing |
| `rand` | 0.8.3 | Random number generation |
| `tokio` | 1.0 (full) | Async runtime |
| `reqwest` | 0.11 (json) | HTTP client for time sync |
| `serde` | 1.0 (derive) | Serialization |
| `serde_json` | 1.0 | JSON encoding |
| `chrono` | 0.4 | Platform-agnostic time handling |

## Network Protocol

Communication uses JSON over TCP with newline-delimited messages.

### Miner Messages
- `GetRoundInfo` - Request current mining round info
- `SubmitBlock` - Submit a mined block
- `CheckLockout` - Check lockout status
- `GetBlockchainInfo` - Get blockchain stats

### Validator Responses
- `RoundInfo` - Current tonce, challenge time, difficulty
- `BlockResult` - Accepted/rejected with reason
- `LockoutStatus` - Locked status and remaining time
- `BlockchainInfo` - Block count, difficulty
- `Error` - Error message

## Implementation Status

- Phase 1: Basic Blockchain (complete)
- Phase 2: Proof of Time Core (complete)
- Phase 3: Network & Distribution (complete)
- Future: Multi-validator consensus, wallets, block explorer, P2P

## Troubleshooting

### Build Errors
```bash
# Clean and rebuild
cargo clean && cargo build --release
```

### Test Failures
- Time-sensitive tests may have timing variance - rerun if marginal failures
- Ensure system clock is reasonably accurate

### Network Issues
- Validator must be running before miners connect
- Default port is 8080 - ensure it's not in use
- Check firewall settings for local testing
