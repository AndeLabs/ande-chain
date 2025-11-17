# MEV Handler Analysis - evstack Pattern

## Overview

evstack implements MEV fee redistribution through a **handler wrapper pattern** that intercepts the `reward_beneficiary()` hook in the EVM execution lifecycle. This allows redirecting base fees to a designated sink address for fair MEV distribution.

## Architecture Pattern

```text
EvHandler (Wrapper)
  ├─ inner: MainnetHandler (Standard execution)
  └─ redirect: Option<BaseFeeRedirect> (MEV policy)
      └─ fee_sink: Address (Distribution contract)

Execution Flow:
Block → EvHandler
  ├─ validate_env()
  ├─ load_accounts()
  ├─ run_exec_loop()
  ├─ reimburse_caller()
  └─ reward_beneficiary() ← INTERCEPTED HERE
      ├─ apply redirect (base_fee * gas_used → sink)
      └─ standard reward (tip → beneficiary)
```

## Key Components

### 1. BaseFeeRedirect (`ev-revm/src/base_fee.rs`)

**Purpose**: Encapsulates MEV redistribution policy

```rust
pub struct BaseFeeRedirect {
    fee_sink: Address,  // Redistribution contract address
}

impl BaseFeeRedirect {
    pub fn apply<CTX>(
        &self,
        ctx: &mut CTX,
        gas_used: u64,
    ) -> Result<U256, BaseFeeRedirectError<<CTX::Db as Database>::Error>>
    where
        CTX: ContextTr,
        CTX::Journal: JournalTr<Database = CTX::Db>,
    {
        let base_fee = ctx.block().basefee();
        let amount = U256::from(base_fee) * U256::from(gas_used);
        
        // Credit sink address with base fee portion
        let journal = ctx.journal_mut();
        journal.load_account(self.fee_sink)?;
        journal.balance_incr(self.fee_sink, amount)?;
        
        Ok(amount)
    }
}
```

**Key features**:
- ✅ Direct journal manipulation for fee redistribution
- ✅ Zero-copy calculation (no intermediate allocations)
- ✅ Error propagation via `BaseFeeRedirectError`
- ✅ Skips redirect if `gas_used == 0` or `base_fee == 0`

### 2. EvHandler (`ev-revm/src/handler.rs`)

**Purpose**: Wrapper over `MainnetHandler` with MEV interception

```rust
pub struct EvHandler<EVM, ERROR, FRAME> {
    inner: MainnetHandler<EVM, ERROR, FRAME>,
    redirect: Option<BaseFeeRedirect>,
}

impl<EVM, ERROR, FRAME> Handler for EvHandler<EVM, ERROR, FRAME>
where
    EVM: EvmTr<Context: ContextTr<Journal: JournalTr<State = EvmState>>, Frame = FRAME>,
    ERROR: EvmTrError<EVM>,
    FRAME: FrameTr<FrameResult = FrameResult, FrameInit = FrameInit>,
{
    // ... all methods delegate to inner ...
    
    fn reward_beneficiary(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <FRAME as FrameTr>::FrameResult,
    ) -> Result<(), Self::Error> {
        let gas = exec_result.gas();
        let spent = gas.spent_sub_refunded();

        // Apply MEV redirect BEFORE standard reward
        if let (Some(redirect), true) = (self.redirect, spent != 0) {
            redirect.apply(evm.ctx(), spent)
                .map_err(|BaseFeeRedirectError::Database(err)| Self::Error::from(err))?;
        }

        // Standard beneficiary reward (priority fee only)
        post_execution::reward_beneficiary(evm.ctx(), gas).map_err(From::from)
    }
}
```

**Key features**:
- ✅ Composition over inheritance (wraps MainnetHandler)
- ✅ Optional redirect (can be disabled)
- ✅ Intercepts ONLY `reward_beneficiary()`
- ✅ All other methods delegated to inner handler
- ✅ Type-safe error handling

### 3. EvEvm (`ev-revm/src/evm.rs`)

**Purpose**: EVM wrapper that carries redirect configuration

```rust
pub struct EvEvm<CTX, INSP, P = ContextPrecompiles<CTX>> {
    inner: Evm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, P, EthFrame<EthInterpreter>>,
    redirect: Option<BaseFeeRedirect>,
    inspect: bool,
}

impl<CTX, INSP, P> EvEvm<CTX, INSP, P> {
    pub fn new(ctx: CTX, inspector: INSP, redirect: Option<BaseFeeRedirect>) -> Self {
        Self {
            inner: Evm { /* ... */ },
            redirect,
            inspect: false,
        }
    }
}
```

## Fee Distribution Flow

```text
Transaction Execution:
  User pays: base_fee + priority_fee (tip)
  
Standard Flow (no MEV):
  Beneficiary receives: base_fee + tip
  
evstack Flow (with redirect):
  1. BaseFeeRedirect.apply() → sink.balance += (base_fee * gas_used)
  2. Beneficiary receives: tip only
  
Result:
  - Sink accumulates ALL base fees
  - Beneficiary gets only tips
  - Sink contract can distribute fairly to validators
```

## Integration Points

### In EVM Factory

evstack integrates in `EvEvmFactory`:

```rust
impl EvmFactory for EvEvmFactory {
    fn evm_with_env_and_inspector<DB, I>(
        &self,
        db: DB,
        env: EnvWithHandlerCfg,
        inspector: I,
    ) -> EvEvm<MainContext<DB>, I>
    where
        DB: Database,
        I: Inspector<MainContext<DB>, EthInterpreter>,
    {
        let redirect = self.base_fee_config
            .map(|cfg| cfg.to_redirect())
            .transpose()?;
            
        EvEvm::new(ctx, inspector, redirect)
    }
}
```

### In Executor

evstack's `EvolveExecutorBuilder` uses `EvEvmFactory` instead of standard `EthEvmFactory`.

## ANDE Adaptation Plan

### Phase 2A: Basic Handler Integration

1. **Create `AndeMevRedirect`** (`ande-evm/src/mev/redirect.rs`)
   - Similar to `BaseFeeRedirect`
   - Add MEV detection hooks (sandwich, arbitrage, liquidation)
   - Track MEV revenue separately from base fees

2. **Create `AndeHandler`** (`ande-evm/src/mev/handler.rs`)
   - Wrap `MainnetHandler`
   - Intercept `reward_beneficiary()`
   - Apply MEV redirect to distribution contract
   - Log MEV events for analytics

3. **Update `AndeEvmFactory`**
   - Add `mev_config: Option<MevConfig>`
   - Create EVM with AndeHandler when config present
   - Backwards compatible (no config = standard handler)

### Phase 2B: MEV Detection

Enhance `reward_beneficiary()` to detect MEV types:

```rust
fn reward_beneficiary(&self, evm, exec_result) -> Result<()> {
    // Detect MEV type
    let mev_type = self.detect_mev(evm.ctx(), exec_result)?;
    
    // Apply appropriate redirect based on MEV type
    if let Some(mev) = mev_type {
        self.mev_redirect.apply(evm.ctx(), mev)?;
    }
    
    // Standard reward
    post_execution::reward_beneficiary(evm.ctx(), gas)?
}
```

### Phase 2C: Distribution Contract

Deploy MEV distribution contract:

```solidity
contract MevDistributor {
    mapping(address => uint256) public validatorShares;
    mapping(address => uint256) public lastClaimBlock;
    
    function distributeMev() external {
        // Calculate fair distribution based on:
        // - Validator stake weight
        // - Blocks proposed
        // - Uptime/performance
    }
    
    function claimRewards() external {
        // Validators claim their share
    }
}
```

## Configuration

```bash
# Enable MEV redistribution
export ANDE_MEV_ENABLED=true
export ANDE_MEV_SINK=0x...  # Distribution contract address

# MEV detection thresholds
export ANDE_MEV_MIN_PROFIT=1000000000000000  # 0.001 ETH
export ANDE_MEV_SANDWICH_THRESHOLD=50000000000000000  # 0.05 ETH
```

## Benefits for ANDE Chain

1. **Fair MEV Distribution**
   - All validators share MEV revenue
   - Prevents centralization of MEV extraction
   - Incentivizes decentralization

2. **Transparency**
   - All MEV redirects logged on-chain
   - Auditable distribution
   - Clear accounting

3. **Compatibility**
   - Optional feature (can be disabled)
   - No breaking changes to standard execution
   - Works with existing tools (Flashbots, Eden, etc.)

4. **Performance**
   - Minimal overhead (single hook interception)
   - Zero-copy fee calculation
   - Efficient journal manipulation

## Testing Strategy

1. **Unit Tests**
   - `AndeMevRedirect::apply()` correctness
   - Fee calculation accuracy
   - Error handling

2. **Integration Tests**
   - End-to-end block execution with MEV
   - Distribution contract interaction
   - Multi-validator scenarios

3. **Mainnet Replay**
   - Test against known MEV transactions
   - Verify detection accuracy
   - Benchmark performance impact

## References

- **evstack pattern**: `/ev-reth-official/crates/ev-revm/src/handler.rs`
- **Reth Handler trait**: `reth_revm::revm::handler::Handler`
- **MEV detection**: Flashbots research papers
- **Distribution models**: Proposer-Builder Separation (PBS)

## Next Steps

1. ✅ Analyze evstack pattern (COMPLETED)
2. ⏳ Implement `AndeMevRedirect` 
3. ⏳ Create `AndeHandler` wrapper
4. ⏳ Integrate in `AndeEvmFactory`
5. ⏳ Deploy MevDistributor contract
6. ⏳ End-to-end testing

---

**Analysis Date**: 2025-01-16  
**Pattern Source**: evstack EvHandler + BaseFeeRedirect  
**Target**: ANDE Chain MEV fair distribution
