//! ANDE EVM Configuration Type Alias
//!
//! This module provides a type alias for ANDE EVM configuration.
//! For now, we use EthEvmConfig directly and inject ANDE precompiles at runtime
//! in the payload builder.

use reth_evm_ethereum::EthEvmConfig;

pub type AndeEvmConfig = EthEvmConfig;


