/// Network module for distributed Hourcoin mining

pub mod protocol;
pub mod validator_server;
pub mod miner_client;

pub use protocol::*;
pub use validator_server::ValidatorServer;
pub use miner_client::MinerClient;
