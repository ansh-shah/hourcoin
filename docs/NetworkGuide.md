### Hourcoin Network Guide - Phase 3: Distributed Mining

## Overview

Phase 3 adds networking capabilities to Hourcoin, enabling distributed mining with multiple miners connecting to a central validator. This transforms Hourcoin from a local demonstration into a functional distributed blockchain system.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  Validator Server                       │
│                                                          │
│  • Maintains canonical blockchain                       │
│  • Enforces Proof of Time consensus                     │
│  • Validates block submissions                          │
│  • Manages miner lockouts                               │
│  • Tracks mining rounds                                 │
│                                                          │
│  TCP Server (127.0.0.1:8080)                           │
└──────────────────┬──────────────────────────────────────┘
                   │
         ┌─────────┴──────────┐
         │                    │
         ▼                    ▼
┌──────────────────┐  ┌──────────────────┐
│  Miner Client 1  │  │  Miner Client 2  │
│                  │  │                  │
│  • Connects      │  │  • Connects      │
│  • Queries tonce │  │  • Queries tonce │
│  • Mines blocks  │  │  • Mines blocks  │
│  • Submits       │  │  • Submits       │
│  • 1hr lockout   │  │  • 1hr lockout   │
└──────────────────┘  └──────────────────┘
```

## Components

### 1. Network Protocol (`src/network/protocol.rs`)

Defines message types exchanged between miners and validators using JSON serialization over TCP.

**Message Types:**

**Miner → Validator:**
- `GetRoundInfo` - Request current mining round information
- `SubmitBlock` - Submit a mined block for validation
- `CheckLockout` - Check if miner is in lockout period
- `GetBlockchainInfo` - Get blockchain statistics

**Validator → Miner:**
- `RoundInfo` - Current round details (tonce, time remaining, etc.)
- `BlockResult` - Result of block submission (accepted/rejected with reason)
- `LockoutStatus` - Miner's lockout status and time remaining
- `BlockchainInfo` - Blockchain statistics
- `Error` - Error message

**Wire Protocol:**
```
[4 bytes: message length (big-endian u32)]
[N bytes: JSON message]
```

### 2. Validator Server (`src/network/validator_server.rs`)

TCP server that manages the blockchain and validates blocks from miners.

**Features:**
- Asynchronous TCP server using Tokio
- Concurrent handling of multiple miner connections
- Thread-safe blockchain access with Arc<Mutex>
- Message-based protocol
- Real-time round management

**Example Usage:**
```rust
use blockchainlib::ValidatorServer;

#[tokio::main]
async fn main() {
    let difficulty = 0x00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
    let address = "127.0.0.1:8080".to_string();

    let mut server = ValidatorServer::new(difficulty, address);
    server.start().await.unwrap();
}
```

### 3. Miner Client (`src/network/miner_client.rs`)

TCP client that connects to validators and performs mining operations.

**Features:**
- Async TCP client
- Automatic tonce challenge solving
- Block mining and submission
- Lockout detection and waiting
- Continuous mining loop

**Example Usage:**
```rust
use blockchainlib::MinerClient;

#[tokio::main]
async fn main() {
    let client = MinerClient::new(
        "miner_alice".to_string(),
        "127.0.0.1:8080".to_string()
    );

    // Get round info
    let info = client.get_round_info().await?;
    println!("Tonce: {}", info.tonce.unwrap());

    // Mine and submit
    client.start_mining(
        vec![0; 32],  // prev_hash
        0,            // index
        difficulty,
        "alice_address"
    ).await?;
}
```

### 4. External Time Synchronization

Enhanced time synchronization with external time source integration.

**Features:**
- World Time API integration
- Automatic fallback to system time
- Configurable tolerance
- Error handling

**API Used:**
- http://worldtimeapi.org/api/timezone/Etc/UTC

**Example:**
```rust
let mut time_sync = TimeSync::new();

// Sync with external source
match time_sync.sync_with_external_source().await {
    Ok(trusted_time) => {
        println!("Synced with: {}", trusted_time.source);
        println!("Timestamp: {}", trusted_time.timestamp_ms);
    }
    Err(e) => {
        eprintln!("Sync failed: {}", e);
    }
}
```

## Running the Network

### Step 1: Start the Validator

```bash
# Build the binaries
cargo build --release

# Start validator with default settings (127.0.0.1:8080)
./target/release/validator

# Or with custom address and difficulty
./target/release/validator 0.0.0.0:8080 0x00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
```

**Output:**
```
=== Hourcoin Validator Server ===

Configuration:
  Address: 127.0.0.1:8080
  Difficulty: 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF

Validator server starting on 127.0.0.1:8080
Waiting for miner connections...
```

### Step 2: Start Miner(s)

In separate terminals, start one or more miners:

```bash
# Miner 1
./target/release/miner alice

# Miner 2 (in another terminal)
./target/release/miner bob

# With custom validator address
./target/release/miner charlie 192.168.1.100:8080 charlie_rewards
```

**Miner Output:**
```
=== Hourcoin Miner Client ===

Configuration:
  Miner ID: alice
  Validator: 127.0.0.1:8080
  Reward Address: alice

Connecting to validator...
✓ Connected to validator

Current Round Info:
  Tonce: 7
  Challenge time remaining: 58 seconds
  Difficulty: 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
  Attempted miners: 0
  Active lockouts: 0

✓ Ready to mine!

Starting continuous mining...
Press Ctrl+C to stop

Mining block #0...
  Tonce: 7
  Challenge time remaining: 58 seconds
  Found valid timestamp: 1762800000123
  ✓ Block mined! Hash: 4a3c2b1e...
  Nonce: 12345
  Submitting to validator...
  ✓ Block accepted! You are now in 1-hour lockout.

⏳ In lockout period. Waiting 3600 seconds...
```

**Validator Output:**
```
New connection from: 127.0.0.1:54321
Miner 'alice' requested round info
Miner 'alice' submitting block #0
✓ Block ACCEPTED from miner 'alice'
  Miner entered 1-hour lockout
  Blockchain now has 1 blocks
```

### Step 3: Watch the Mining

Multiple miners will compete in each round:
- Only one miner can successfully mine per round
- Successful miner enters 1-hour lockout
- Other miners continue competing in next round
- New tonce generated for each round

## Network Protocol Details

### Message Format

All messages are JSON-encoded and prefixed with a 4-byte length field.

**Example GetRoundInfo Request:**
```json
{
  "GetRoundInfo": {
    "miner_id": "alice"
  }
}
```

**Example RoundInfo Response:**
```json
{
  "RoundInfo": {
    "round_start": 1762800000000,
    "tonce": 7,
    "challenge_seconds_remaining": 58,
    "attempted_miners": 2,
    "active_lockouts": 1,
    "difficulty": "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
  }
}
```

**Example SubmitBlock Request:**
```json
{
  "SubmitBlock": {
    "miner_id": "alice",
    "block": {
      "index": 0,
      "timestamp": 1762800000123,
      "hash": "4a3c2b1e...",
      "prev_block_hash": "0000...",
      "nonce": 12345,
      "transactions": [...]
    }
  }
}
```

**Example BlockResult Response:**
```json
{
  "BlockResult": {
    "result": "Accepted",
    "message": "Block accepted! You are now in 1-hour lockout."
  }
}
```

### Error Handling

The protocol handles various error conditions:

1. **Network Errors**: Automatic reconnection on disconnect
2. **Validation Errors**: Descriptive error messages
3. **Timeout Errors**: Configurable timeouts
4. **Parse Errors**: JSON validation

**Example Error Response:**
```json
{
  "Error": {
    "message": "Invalid block data: Hash decode failed"
  }
}
```

## Testing the Network

### Unit Tests

```bash
cargo test
```

All 51 tests should pass, including new network protocol tests.

### Integration Test

1. Start validator:
```bash
./target/release/validator
```

2. In another terminal, start a miner:
```bash
./target/release/miner test_miner
```

3. Verify:
   - Miner connects successfully
   - Round info is retrieved
   - Block is mined and submitted
   - Validator accepts/rejects appropriately
   - Lockout is enforced

### Multi-Miner Test

1. Start validator
2. Start 3+ miners simultaneously:
```bash
./target/release/miner alice &
./target/release/miner bob &
./target/release/miner charlie &
```

3. Observe:
   - Only one miner succeeds per round
   - Others are rejected (already attempted)
   - Successful miner enters lockout
   - New round starts with new tonce

## Configuration

### Validator Configuration

```bash
# Syntax
./target/release/validator [address] [difficulty]

# Examples
./target/release/validator
./target/release/validator 0.0.0.0:8080
./target/release/validator 0.0.0.0:8080 0x00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
```

### Miner Configuration

```bash
# Syntax
./target/release/miner [miner_id] [validator_address] [reward_address]

# Examples
./target/release/miner
./target/release/miner alice
./target/release/miner alice 192.168.1.100:8080
./target/release/miner alice 192.168.1.100:8080 alice_rewards
```

## Performance Considerations

### Network Performance

- **Connection Overhead**: Each miner maintains one TCP connection
- **Message Size**: Typical messages are < 1KB
- **Latency**: Tonce validation includes network round-trip time
- **Throughput**: Validator can handle 100+ concurrent miners

### Scalability

**Current Limitations:**
- Single validator (centralized)
- No load balancing
- In-memory blockchain storage
- TCP connection per miner

**Future Improvements:**
- Multiple validator nodes
- Distributed consensus among validators
- Persistent blockchain storage
- WebSocket support for real-time updates

## Security Considerations

### Network Security

**Current Implementation:**
- Plain TCP (no encryption)
- No authentication
- No rate limiting
- Basic input validation

**Production Requirements:**
- TLS/SSL for encrypted communication
- Miner authentication (API keys, certificates)
- Rate limiting per IP/miner
- DDoS protection
- Input sanitization

### Consensus Security

**Attack Vectors:**
- Sybil attacks (multiple identities)
- Time manipulation
- Network flooding
- Block withholding

**Mitigations:**
- IP-based tracking (planned)
- External time validation
- Connection limits
- Economic disincentives (1-hour lockout)

## Troubleshooting

### Validator Won't Start

```
Error: Address already in use
```
**Solution:** Another process is using port 8080. Stop it or use a different port.

### Miner Can't Connect

```
Failed to connect to validator
```
**Solution:** Verify validator is running and address is correct.

### Block Always Rejected

```
Block rejected: Timestamp failed tonce challenge
```
**Solution:** The tonce challenge is hard. Increase max_attempts in find_valid_timestamp.

### Lockout Not Working

```
Miner submits blocks immediately after success
```
**Solution:** Check validator's lockout tracking. Ensure miner IDs are unique.

## API Reference

### ValidatorServer

```rust
pub struct ValidatorServer {
    // Creates new validator server
    pub fn new(difficulty: u128, address: String) -> Self;

    // Starts the server (blocks)
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}
```

### MinerClient

```rust
pub struct MinerClient {
    // Creates new miner client
    pub fn new(miner_id: String, validator_address: String) -> Self;

    // Gets current round information
    pub async fn get_round_info(&self) -> Result<RoundInfoData, Box<dyn std::error::Error>>;

    // Checks lockout status
    pub async fn check_lockout(&self) -> Result<(bool, u64), Box<dyn std::error::Error>>;

    // Mines and submits a single block
    pub async fn mine_and_submit(
        &self,
        prev_hash: Vec<u8>,
        index: u32,
        difficulty: u128,
        reward_address: &str,
    ) -> Result<ValidatorMessage, Box<dyn std::error::Error>>;

    // Starts continuous mining loop
    pub async fn start_mining(
        &self,
        initial_prev_hash: Vec<u8>,
        initial_index: u32,
        difficulty: u128,
        reward_address: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
```

## Next Steps

With Phase 3 complete, Hourcoin now supports distributed mining. Future enhancements include:

1. **Multi-Validator Consensus**: Multiple validators with Byzantine fault tolerance
2. **Persistent Storage**: Database backend for blockchain
3. **Web Dashboard**: Real-time monitoring of network
4. **Wallet System**: Public/private key cryptography
5. **P2P Network**: Peer-to-peer communication (no central validator)
6. **Smart Contracts**: Programmable transactions

## Summary

Phase 3 successfully implements:
- ✅ Network protocol for miner-validator communication
- ✅ Validator TCP server with concurrent connection handling
- ✅ Miner client with automatic mining
- ✅ External time synchronization
- ✅ Standalone validator binary
- ✅ Standalone miner binary
- ✅ 51 passing tests

Hourcoin is now a functional distributed blockchain with Proof of Time consensus!
