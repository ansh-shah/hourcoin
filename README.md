# hourcoin
A cryptocurrency to value time...

## What is Hourcoin?

Hourcoin is an experimental blockchain that implements a novel **Proof of Time** consensus mechanism. Unlike traditional Proof-of-Work cryptocurrencies that value computational power, Hourcoin values miners' time - each coin represents one hour of a miner's life.

## Core Concept: Proof of Time

The Proof of Time consensus mechanism consists of:

1. **Tonce Challenges**: Time-based mining puzzles where miners must find timestamps whose hash is divisible by a random value (tonce)
2. **Miner Sacrifice Protocol**: Miners who successfully mine a block must wait 1 hour before mining again
3. **Fair Mining**: The time-based challenges level the playing field - it's not about hardware, it's about time

## Implementation Status

✅ **Phase 1: Basic Blockchain** (COMPLETED)
- Rust-based blockchain with UTXO model
- Proof-of-Work mining with adjustable difficulty
- Transaction validation with timestamp checks
- Comprehensive test suite (47 tests passing)

✅ **Phase 2: Proof of Time Core** (COMPLETED)
- Time synchronization module
- Tonce (time-only-used-once) system
- Validator/timekeeper node
- Miner sacrifice protocol with 1-hour lockout
- Working demonstration programs

✅ **Phase 3: Network & Distribution** (COMPLETED)
- TCP-based networking protocol
- Validator server with concurrent miner support
- Miner client with automatic mining
- External time synchronization (World Time API)
- Standalone validator and miner binaries
- JSON message protocol over TCP
- Comprehensive test suite (51 tests passing)

🔄 **Future Enhancements**
- Multi-validator consensus
- Wallet and address system
- Block explorer web interface
- P2P networking

## Quick Start

### Run the Demo
```bash
cargo run --release
```

This demonstrates both traditional PoW and the new Proof of Time consensus.

### Run Full Proof of Time Demo
```bash
cargo run --release --example proof_of_time_demo
```

### Run Tests
```bash
cargo test
```

All 51 tests should pass.

### Run Distributed Mining Network

**Step 1: Start the Validator Server**
```bash
cargo build --release
./target/release/validator
```

**Step 2: Start Miner(s)** (in separate terminals)
```bash
./target/release/miner alice
./target/release/miner bob
./target/release/miner charlie
```

Miners will automatically connect to the validator, mine blocks, and compete in each round!

See [Network Guide](./docs/NetworkGuide.md) for detailed instructions.

## Documentation

- [Roadmap](https://github.com/Nit123/hourcoin/blob/main/docs/Roadmap.md) - Project development plan
- [Proof of Time Documentation](./docs/ProofOfTime.md) - Core consensus mechanism
  - Architecture overview
  - How tonce works
  - Miner sacrifice protocol
  - Validator operations
  - API reference
  - Security considerations
- [Network Guide](./docs/NetworkGuide.md) - Distributed mining setup
  - Network architecture
  - Running validator server
  - Running miner clients
  - Protocol specifications
  - Configuration options
  - Troubleshooting

## Example: How Proof of Time Works

```rust
// 1. Validator starts a new mining round
let mut validator = Validator::new(difficulty);
validator.start_new_round();

// 2. Validator calculates tonce from previous block
let tonce = validator.get_current_tonce().unwrap();
println!("Tonce for this round: {}", tonce);

// 3. Miner finds a timestamp that satisfies the tonce
let valid_timestamp = find_valid_timestamp(tonce, now(), 10000)?;

// 4. Miner creates and mines block
let mut block = Block::new(index, valid_timestamp, prev_hash, transactions);
block.mine(difficulty);

// 5. Miner submits to validator
let result = validator.validate_block_submission(block, "miner_id".to_string());

// 6. If accepted, miner enters 1-hour lockout
match result {
    ValidationResult::Accepted => {
        println!("Block accepted! Miner locked out for 1 hour");
        println!("Earned 2 hourcoin (1 for work, 1 for time sacrifice)");
    }
    ValidationResult::RejectedMinerInLockout => {
        println!("Miner still locked out from previous block");
    }
    ValidationResult::RejectedTonceChallenge => {
        println!("Timestamp didn't pass tonce challenge");
    }
    _ => println!("Block rejected: {:?}", result),
}
```

## Key Features

- **Time-Based Mining**: Tonce challenges require miners to find timestamps with specific hash properties
- **Fair Distribution**: All miners have equal opportunity regardless of hardware
- **Miner Sacrifice**: One hour lockout enforces the "value of time" concept
- **Randomized Difficulty**: Each round has a different tonce (1-31)
- **60-Second Challenge**: First 60 seconds require tonce solution, then becomes a race
- **2 Hourcoin Reward**: 1 for validator work, 1 for time sacrifice

## Project Structure

```
src/
├── lib.rs              # Library exports
├── main.rs             # Demo program
├── block.rs            # Block structure and PoW mining
├── blockchain.rs       # Blockchain validation
├── transaction.rs      # Transaction handling
├── time_sync.rs        # Time synchronization
├── tonce.rs            # Tonce challenge system
└── validator.rs        # Validator/timekeeper node

examples/
└── proof_of_time_demo.rs  # Complete PoT demo

docs/
├── Roadmap.md          # Development roadmap
└── ProofOfTime.md      # Technical documentation
```

## Why Hourcoin?

Traditional cryptocurrencies require expensive hardware and consume massive amounts of electricity. Hourcoin takes a different approach:

- **Accessible**: No need for specialized mining hardware
- **Sustainable**: Time-based consensus uses minimal energy
- **Fair**: All participants have 24 hours in a day
- **Meaningful**: Each coin represents an hour of human time

## Contributing

This is an experimental educational project. Contributions and feedback are welcome!

See the [Roadmap](./docs/Roadmap.md) for planned features and development goals.

## License

MIT

## Credits

Created by Nitesh with contributions from Ansh Shah.

Inspired by [TinyCoin](https://github.com/JeremyRubin/tinycoin) and research into time-based consensus mechanisms.
