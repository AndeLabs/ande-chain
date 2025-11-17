//! MEV execution handler for ANDE Chain
//!
//! Implements handler wrapper that intercepts `reward_beneficiary` hook
//! to redirect MEV profits to fair distribution contract.
//!
//! ## Pattern
//!
//! Follows evstack's EvHandler pattern:
//! - Wraps MainnetHandler
//! - Intercepts reward_beneficiary()
//! - Applies MEV redirect before standard reward
//! - All other methods delegated to inner handler

use crate::mev::redirect::{AndeMevRedirect, MevRedirectError};
use reth_revm::{
    inspector::{Inspector, InspectorEvmTr, InspectorHandler},
    revm::{
        context::result::ExecutionResult,
        context_interface::{result::HaltReason, ContextTr, JournalTr},
        handler::{
            post_execution, EthFrame, EvmTr, EvmTrError, FrameResult, FrameTr, Handler,
            MainnetHandler,
        },
        interpreter::{
            interpreter::EthInterpreter, interpreter_action::FrameInit, InitialAndFloorGas,
        },
        state::EvmState,
    },
};
use tracing::{debug, info};

/// Handler wrapper that applies ANDE-specific MEV distribution policies.
///
/// This handler mirrors the standard mainnet handler but intercepts the
/// `reward_beneficiary` hook to redirect MEV profits to a distribution contract.
#[derive(Debug, Clone)]
pub struct AndeHandler<EVM, ERROR, FRAME> {
    /// Inner mainnet handler for standard execution
    inner: MainnetHandler<EVM, ERROR, FRAME>,
    /// MEV redirect policy (optional)
    mev_redirect: Option<AndeMevRedirect>,
}

impl<EVM, ERROR, FRAME> AndeHandler<EVM, ERROR, FRAME> {
    /// Creates a new ANDE handler with the provided MEV redirect policy.
    ///
    /// # Arguments
    ///
    /// * `mev_redirect` - Optional MEV distribution policy. If `None`, behaves like standard handler.
    pub fn new(mev_redirect: Option<AndeMevRedirect>) -> Self {
        if mev_redirect.is_some() {
            info!("🎯 AndeHandler initialized with MEV redistribution enabled");
        } else {
            debug!("AndeHandler initialized in standard mode (no MEV redirect)");
        }
        
        Self {
            inner: MainnetHandler::default(),
            mev_redirect,
        }
    }

    /// Returns the configured MEV redirect policy, if any.
    pub const fn mev_redirect(&self) -> Option<AndeMevRedirect> {
        self.mev_redirect
    }
}

impl<EVM, ERROR, FRAME> Handler for AndeHandler<EVM, ERROR, FRAME>
where
    EVM: EvmTr<Context: ContextTr<Journal: JournalTr<State = EvmState>>, Frame = FRAME>,
    ERROR: EvmTrError<EVM>,
    FRAME: FrameTr<FrameResult = FrameResult, FrameInit = FrameInit>,
{
    type Evm = EVM;
    type Error = ERROR;
    type HaltReason = HaltReason;

    fn validate_env(&self, evm: &mut Self::Evm) -> Result<(), Self::Error> {
        self.inner.validate_env(evm)
    }

    fn validate_initial_tx_gas(&self, evm: &Self::Evm) -> Result<InitialAndFloorGas, Self::Error> {
        self.inner.validate_initial_tx_gas(evm)
    }

    fn load_accounts(&self, evm: &mut Self::Evm) -> Result<(), Self::Error> {
        self.inner.load_accounts(evm)
    }

    fn apply_eip7702_auth_list(&self, evm: &mut Self::Evm) -> Result<u64, Self::Error> {
        self.inner.apply_eip7702_auth_list(evm)
    }

    fn validate_against_state_and_deduct_caller(
        &self,
        evm: &mut Self::Evm,
    ) -> Result<(), Self::Error> {
        self.inner.validate_against_state_and_deduct_caller(evm)
    }

    fn first_frame_input(
        &mut self,
        evm: &mut Self::Evm,
        gas_limit: u64,
    ) -> Result<FRAME::FrameInit, Self::Error> {
        self.inner.first_frame_input(evm, gas_limit)
    }

    fn last_frame_result(
        &mut self,
        evm: &mut Self::Evm,
        frame_result: &mut <FRAME as FrameTr>::FrameResult,
    ) -> Result<(), Self::Error> {
        self.inner.last_frame_result(evm, frame_result)
    }

    fn run_exec_loop(
        &mut self,
        evm: &mut Self::Evm,
        first_frame_input: <FRAME as FrameTr>::FrameInit,
    ) -> Result<FrameResult, Self::Error> {
        self.inner.run_exec_loop(evm, first_frame_input)
    }

    fn eip7623_check_gas_floor(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <FRAME as FrameTr>::FrameResult,
        init_and_floor_gas: InitialAndFloorGas,
    ) {
        self.inner
            .eip7623_check_gas_floor(evm, exec_result, init_and_floor_gas)
    }

    fn refund(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <FRAME as FrameTr>::FrameResult,
        eip7702_refund: i64,
    ) {
        self.inner.refund(evm, exec_result, eip7702_refund)
    }

    fn reimburse_caller(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <FRAME as FrameTr>::FrameResult,
    ) -> Result<(), Self::Error> {
        self.inner.reimburse_caller(evm, exec_result)
    }

    /// Rewards the block beneficiary and applies MEV redistribution.
    ///
    /// This is the KEY hook where MEV detection and redistribution happens.
    ///
    /// Flow:
    /// 1. Calculate gas spent
    /// 2. If MEV redirect configured AND gas > 0:
    ///    a. Apply MEV redirect (base_fee * gas → mev_sink)
    ///    b. Log MEV detection
    /// 3. Standard beneficiary reward (tips)
    fn reward_beneficiary(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <FRAME as FrameTr>::FrameResult,
    ) -> Result<(), Self::Error> {
        let gas = exec_result.gas();
        let spent = gas.spent_sub_refunded();

        // Apply MEV redirect if configured and gas was spent
        if let (Some(redirect), true) = (self.mev_redirect, spent != 0) {
            match redirect.apply(evm.ctx(), spent) {
                Ok(detection) => {
                    // Log MEV detection for analytics
                    debug!(
                        mev_type = ?detection.mev_type,
                        profit = %detection.profit,
                        gas_used = detection.gas_used,
                        "MEV redirect applied"
                    );
                    
                    if !detection.profit.is_zero() {
                        info!(
                            mev_sink = ?redirect.mev_sink(),
                            profit = %detection.profit,
                            "💰 MEV profit redirected to distribution contract"
                        );
                    }
                }
                Err(MevRedirectError::Database(err)) => {
                    // Convert database error to handler error
                    return Err(Self::Error::from(err));
                }
            }
        }

        // Apply standard beneficiary reward (priority fees/tips)
        post_execution::reward_beneficiary(evm.ctx(), gas).map_err(From::from)
    }

    fn execution_result(
        &mut self,
        evm: &mut Self::Evm,
        result: <FRAME as FrameTr>::FrameResult,
    ) -> Result<ExecutionResult<Self::HaltReason>, Self::Error> {
        self.inner.execution_result(evm, result)
    }
}

/// Inspector handler support for ANDE handler.
///
/// This allows the handler to work with inspector-enabled EVMs.
impl<EVM, ERROR> InspectorHandler for AndeHandler<EVM, ERROR, EthFrame<EthInterpreter>>
where
    EVM: InspectorEvmTr<
        Context: ContextTr<Journal: JournalTr<State = EvmState>>,
        Frame = EthFrame<EthInterpreter>,
        Inspector: Inspector<<EVM as EvmTr>::Context, EthInterpreter>,
    >,
    ERROR: EvmTrError<EVM>,
{
    type IT = EthInterpreter;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mev::MevType;
    use alloy_primitives::{address, Address, Bytes, U256};
    use reth_revm::{
        inspector::NoOpInspector,
        revm::{
            context::Context,
            context::{BlockEnv, CfgEnv, TxEnv},
            database::EmptyDB,
            handler::EthFrame,
            interpreter::{CallOutcome, Gas, InstructionResult, InterpreterResult},
            primitives::hardfork::SpecId,
        },
    };
    use std::convert::Infallible;

    use reth_revm::revm::context_interface::result::{EVMError, InvalidTransaction};

    type TestContext = Context<BlockEnv, TxEnv, CfgEnv<SpecId>, EmptyDB>;
    // We'll define a test EVM type when needed
    type TestError = EVMError<Infallible, InvalidTransaction>;

    const BASE_FEE: u64 = 100;
    const GAS_PRICE: u128 = 200;
    const GAS_USED: u64 = 21_000;

    #[test]
    fn test_handler_creation_with_redirect() {
        let sink = address!("0x0000000000000000000000000000000000000001");
        let redirect = AndeMevRedirect::with_default_threshold(sink);
        
        let handler = AndeHandler::<(), TestError, EthFrame<EthInterpreter>>::new(Some(redirect));
        
        assert!(handler.mev_redirect().is_some());
        assert_eq!(handler.mev_redirect().unwrap().mev_sink(), sink);
    }

    #[test]
    fn test_handler_creation_without_redirect() {
        let handler = AndeHandler::<(), TestError, EthFrame<EthInterpreter>>::new(None);
        
        assert!(handler.mev_redirect().is_none());
    }

    // Note: Full integration tests for reward_beneficiary require a complete EVM setup
    // which is complex. The redirect logic is already tested in redirect.rs
}
