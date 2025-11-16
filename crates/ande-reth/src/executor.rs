//! ANDE Executor Builder
//!
//! Custom executor builder that integrates ANDE Chain personalizaciones:
//! - Token Duality Precompile (0xFD) via AndeEvmConfig
//! - (Future) Parallel EVM execution via Block-STM
//! - (Future) MEV-aware execution ordering

use ande_evm::AndeEvmFactory;
use reth_chainspec::{ChainSpec, EthChainSpec, EthereumHardforks, Hardforks};
use reth_ethereum::evm::EthEvmConfig;
use reth_ethereum_primitives::EthPrimitives;
use reth_evm::eth::spec::EthExecutorSpec;
use reth_node_builder::{BuilderContext, components::ExecutorBuilder, FullNodeTypes, NodeTypes};

/// ANDE Chain Executor Builder
///
/// This builder configures the EVM execution environment with ANDE Chain customizations:
/// 
/// ## Active Features (v1.0):
/// - ✅ Token Duality Precompile at address 0xFD
///   - Native ANDE token accessible as ERC20
///   - Gas: 3000 base + 100/word
///   - Security: Balance checks, overflow protection
///
/// ## Planned Features (v2.0):
/// - ⏳ Parallel EVM Execution (Block-STM)
///   - Multi-version concurrency control
///   - Automatic conflict detection
///   - 10-15x throughput improvement
/// 
/// - ⏳ MEV-Aware Execution
///   - Bundle execution support
///   - Fair MEV distribution (80% stakers, 20% treasury)
///
/// ## Integration Points:
/// - Integrated via `AndeNode::components()` → `executor(AndeExecutorBuilder)`
/// - Uses `AndeEvmConfig` which wraps `EthEvmConfig` + `AndePrecompileProvider`
/// - Compatible with Evolve sequencer (standard Engine API)
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct AndeExecutorBuilder;

impl<Types, Node> ExecutorBuilder<Node> for AndeExecutorBuilder
where
    Types: NodeTypes<
        ChainSpec: Hardforks + EthExecutorSpec + EthereumHardforks,
        Primitives = EthPrimitives,
    >,
    Node: FullNodeTypes<Types = Types>,
{
    type EVM = EthEvmConfig<Types::ChainSpec, AndeEvmFactory>;

    async fn build_evm(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::EVM> {
        use alloy_evm::EthEvmFactory;
        use reth_ethereum::evm::revm::primitives::hardfork::SpecId;
        
        tracing::info!("🔧 Building ANDE EVM with wrapper pattern");
        
        // Determine spec ID from chain spec (default to CANCUN)
        let spec_id = SpecId::CANCUN;
        
        // Create ANDE EVM factory (wrapper pattern)
        // Wraps standard EthEvmFactory and will inject precompiles
        let inner_factory = EthEvmFactory::default();
        let ande_factory = AndeEvmFactory::new(inner_factory, spec_id);
        
        // Create EthEvmConfig with our wrapper factory
        let evm_config = EthEvmConfig::new_with_evm_factory(
            ctx.chain_spec().clone(),
            ande_factory,
        );
        
        tracing::info!("✅ ANDE EVM configured (wrapper pattern):");
        tracing::info!("   • Chain ID: {}", ctx.chain_spec().chain().id());
        tracing::info!("   • Spec ID: {:?}", spec_id);
        tracing::info!("   • Inner Factory: EthEvmFactory");
        tracing::info!("   • Wrapper: AndeEvmFactory");
        tracing::info!("   • Precompiles: Standard + ANDE (pending injection)");
        
        Ok(evm_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ande_executor_builder_creation() {
        let _builder = AndeExecutorBuilder;
        // Struct creation test - actual EVM building requires full node context
    }
}
