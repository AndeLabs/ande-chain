# AndeChain Governor v2 Upgrade - Manual Process

## 📋 Overview

**Current:** AndeGovernor v1 (Proxy: `0xB7f8BC63BbcaD18155201308C8f3540b07f84F5e`)
**Target:** AndeGovernor v2 with Security Extensions
**Method:** UUPS Manual Upgrade via Governance
**Timeline:** 2-4 weeks after testnet validation

---

## 🛡️ What Security Extensions Add

### 1. Anti-Whale Protection 🐋
- **Caps voting power at 10% of total supply**
- Prevents single addresses from controlling governance
- Applies to both token votes and staking bonus

### 2. Rate Limiting ⏱️
- **1 day cooldown between proposals** from same address
- Prevents proposal spam attacks
- Reduces governance fatigue

### 3. Guardian Role 🛡️
- **Emergency cancellation** of malicious proposals
- Multisig-controlled (5/7 recommended)
- Can be revoked by governance if needed

### 4. Anti-Frontrunning 🏃
- Proposer commitment mechanism
- Reduces MEV attacks on governance

---

## ⚠️ Critical: Why We Need This

**Without Security Extensions:**
- ❌ Whale with 15% supply controls governance
- ❌ Attacker can spam 100 proposals/day
- ❌ No emergency stop for malicious proposals
- ❌ Governance vulnerable to frontrunning

**With Security Extensions:**
- ✅ Max 10% voting power per address
- ✅ Max 1 proposal/day per address
- ✅ Guardian can cancel attacks
- ✅ Frontrunning protection

---

## 📊 Storage Compatibility Check

### v1 Storage Layout (MUST PRESERVE)
```
Slot 0-10:  GovernorUpgradeable base
Slot 11-13: GovernorSettings (_votingDelay, _votingPeriod, _proposalThreshold)
Slot 14:    GovernorDualTokenVoting (stakingContract)
Slot 15-27: GovernorAdaptiveQuorum (history array + counters)
Slot 28-30: GovernorMultiLevel (mappings + emergencyCouncil)
Slot 31:    GovernorTimelockControl (_timelock)
Slot 32-81: __gap (50 slots reserved)
```

### v2 Storage Layout (APPEND ONLY)
```
Slot 0-31:  UNCHANGED (all v1 storage preserved)
Slot 32:    lastProposalTime mapping (NEW)
Slot 33:    guardian address (NEW)
Slot 34:    proposalCommitment mapping (NEW)
Slot 35-81: __gap_v2 (47 slots, reduced from 50)
```

✅ **Compatible:** All v1 storage slots preserved, v2 storage appended at end using gap

---

## 🔧 Manual Upgrade Process

### PHASE 1: Create v2 Contract (Week 1)

#### Step 1.1: Create AndeGovernorV2.sol

File: `src/governance/AndeGovernorV2.sol`

**Changes from v1:**
1. Add `GovernorSecurityExtensions` to inheritance list
2. Add new storage variables (lastProposalTime, guardian, proposalCommitment)
3. Update gap from 50 to 47 slots
4. Add `initializeV2(address _guardian)` function
5. Add overrides for propose() and _castVote()
6. Add version() function returning "2.0.0"

#### Step 1.2: Compile with Size Optimization

```bash
cd andechain/contracts

# Clean previous builds
forge clean

# Build with extreme optimization
forge build --optimizer-runs 1 --skip test

# Check size
BYTECODE=$(jq -r '.bytecode.object' out/AndeGovernorV2.sol/AndeGovernorV2.json)
SIZE_BYTES=$((${#BYTECODE} / 2))
echo "Size: $SIZE_BYTES bytes (limit: 24576)"

# Must be < 24576 bytes
```

**Expected size:** ~23,800 bytes (within 24KB limit)

#### Step 1.3: Deploy v2 Implementation

```bash
# Get bytecode
BYTECODE=$(jq -r '.bytecode.object' out/AndeGovernorV2.sol/AndeGovernorV2.json)

# Deploy
cast send --rpc-url http://localhost:8545 \
  --private-key $DEPLOYER_KEY \
  --legacy \
  --create "$BYTECODE"

# Save the deployed address
IMPL_V2=0x... (from deployment output)

# Verify deployment
cast code $IMPL_V2 --rpc-url http://localhost:8545
```

---

### PHASE 2: Test Everything (Week 2)

#### Step 2.1: Test on Local Fork

```bash
# 1. Fork current testnet state
anvil --fork-url http://localhost:8545 --port 8546

# 2. Deploy v2 implementation on fork
cast send --rpc-url http://localhost:8546 ...

# 3. Manually test upgrade
# - Upgrade proxy to v2
# - Call initializeV2()
# - Test all v1 features still work
# - Test all v2 features work
```

#### Step 2.2: Manual Verification Checklist

**v1 Features (must still work):**
```bash
# Check voting delay
cast call $GOVERNOR "votingDelay()(uint256)" --rpc-url $RPC

# Check voting period
cast call $GOVERNOR "votingPeriod()(uint256)" --rpc-url $RPC

# Check token address
cast call $GOVERNOR "token()(address)" --rpc-url $RPC

# Check staking contract
cast call $GOVERNOR "stakingContract()(address)" --rpc-url $RPC

# Check timelock
cast call $GOVERNOR "timelock()(address)" --rpc-url $RPC

# Create test proposal (should work)
cast send $GOVERNOR "propose(...)" --rpc-url $RPC
```

**v2 Features (new):**
```bash
# Check version
cast call $GOVERNOR "version()(string)" --rpc-url $RPC
# Expected: "2.0.0"

# Check guardian set
cast call $GOVERNOR "guardian()(address)" --rpc-url $RPC
# Expected: guardian multisig address

# Check MAX_VOTING_POWER_BPS
cast call $GOVERNOR "MAX_VOTING_POWER_BPS()(uint256)" --rpc-url $RPC
# Expected: 1000 (10%)

# Test rate limiting
# - Create proposal
# - Try creating another immediately (should fail)
# - Check lastProposalTime mapping

# Test anti-whale
# - User with >10% supply votes
# - Verify vote capped at 10%
```

#### Step 2.3: Security Review

**Manual checks:**
- [ ] Storage layout matches v1 exactly for first 31 slots
- [ ] New storage appended at end using gap
- [ ] No storage collisions
- [ ] All access controls correct
- [ ] Guardian powers limited to cancel only
- [ ] Rate limiting enforced
- [ ] Anti-whale cap applied correctly

---

### PHASE 3: Setup Guardian Multisig (Week 2)

#### Step 3.1: Create Guardian Multisig

**Recommended: Gnosis Safe 5/7**

```
Members:
- 0xAddress1 (Core Team Lead)
- 0xAddress2 (Core Team Security)
- 0xAddress3 (Core Team Dev)
- 0xAddress4 (Community Rep 1)
- 0xAddress5 (Community Rep 2)
- 0xAddress6 (Security Advisor 1)
- 0xAddress7 (Security Advisor 2)

Threshold: 5/7 (71% consensus)
```

#### Step 3.2: Test Guardian Multisig

```bash
# Test creating multisig transaction
# Test signature collection (5/7)
# Test execution
# Verify all signers can sign
```

---

### PHASE 4: Governance Proposal (Week 3)

#### Step 4.1: Prepare Upgrade Calldata

```bash
# 1. Get addresses
GOVERNOR_PROXY=0xB7f8BC63BbcaD18155201308C8f3540b07f84F5e
IMPL_V2=0x... (from Phase 1)
GUARDIAN_MULTISIG=0x... (from Phase 3)

# 2. Encode initializeV2 calldata
INIT_DATA=$(cast calldata "initializeV2(address)" $GUARDIAN_MULTISIG)

# 3. Encode upgradeToAndCall
UPGRADE_CALLDATA=$(cast calldata "upgradeToAndCall(address,bytes)" $IMPL_V2 $INIT_DATA)

# 4. Prepare proposal arrays
TARGETS=[$GOVERNOR_PROXY]
VALUES=[0]
CALLDATAS=[$UPGRADE_CALLDATA]
DESCRIPTION="Upgrade AndeGovernor to v2 with Security Extensions: Anti-whale (10% cap), Rate limiting (1 day cooldown), Guardian emergency cancel, Anti-frontrunning protection"
```

#### Step 4.2: Create Proposal

```bash
# Encode propose calldata
PROPOSE_DATA=$(cast calldata "propose(address[],uint256[],bytes[],string)" \
  "[$GOVERNOR_PROXY]" \
  "[0]" \
  "[$UPGRADE_CALLDATA]" \
  "$DESCRIPTION")

# Submit proposal
cast send $GOVERNOR_PROXY \
  --rpc-url http://localhost:8545 \
  --private-key $PROPOSER_KEY \
  --legacy \
  "propose(address[],uint256[],bytes[],string)" \
  "[$GOVERNOR_PROXY]" \
  "[0]" \
  "[$UPGRADE_CALLDATA]" \
  "$DESCRIPTION"

# Get proposal ID from logs
PROPOSAL_ID=... (from event ProposalCreated)
```

#### Step 4.3: Monitor Voting

```bash
# Check proposal state
cast call $GOVERNOR "state(uint256)(uint8)" $PROPOSAL_ID --rpc-url $RPC

# States:
# 0 = Pending
# 1 = Active
# 2 = Canceled
# 3 = Defeated
# 4 = Succeeded
# 5 = Queued
# 6 = Expired
# 7 = Executed

# Check votes
cast call $GOVERNOR "proposalVotes(uint256)(uint256,uint256,uint256)" $PROPOSAL_ID --rpc-url $RPC
# Returns: (againstVotes, forVotes, abstainVotes)

# Check if quorum reached
BLOCK=$(cast call $GOVERNOR "proposalSnapshot(uint256)(uint256)" $PROPOSAL_ID --rpc-url $RPC)
QUORUM=$(cast call $GOVERNOR "quorum(uint256)(uint256)" $BLOCK --rpc-url $RPC)
echo "Quorum required: $QUORUM"
```

#### Step 4.4: Queue Proposal

**After voting succeeds (state = 4):**

```bash
# Queue in timelock
cast send $GOVERNOR \
  --rpc-url http://localhost:8545 \
  --private-key $EXECUTOR_KEY \
  --legacy \
  "queue(address[],uint256[],bytes[],bytes32)" \
  "[$GOVERNOR_PROXY]" \
  "[0]" \
  "[$UPGRADE_CALLDATA]" \
  $(cast keccak "$DESCRIPTION")

# Verify queued
cast call $GOVERNOR "state(uint256)(uint8)" $PROPOSAL_ID --rpc-url $RPC
# Should return 5 (Queued)
```

#### Step 4.5: Execute Upgrade

**After timelock delay (1 hour):**

```bash
# Execute
cast send $GOVERNOR \
  --rpc-url http://localhost:8545 \
  --private-key $EXECUTOR_KEY \
  --legacy \
  "execute(address[],uint256[],bytes[],bytes32)" \
  "[$GOVERNOR_PROXY]" \
  "[0]" \
  "[$UPGRADE_CALLDATA]" \
  $(cast keccak "$DESCRIPTION")

# Verify executed
cast call $GOVERNOR "state(uint256)(uint8)" $PROPOSAL_ID --rpc-url $RPC
# Should return 7 (Executed)
```

---

### PHASE 5: Post-Upgrade Verification (Week 3-4)

#### Step 5.1: Immediate Checks

```bash
# 1. Verify version
cast call $GOVERNOR "version()(string)" --rpc-url $RPC
# Must return: "2.0.0"

# 2. Verify guardian
cast call $GOVERNOR "guardian()(address)" --rpc-url $RPC
# Must return: guardian multisig address

# 3. Verify v1 features unchanged
cast call $GOVERNOR "votingPeriod()(uint256)" --rpc-url $RPC
# Must return: 21600

cast call $GOVERNOR "token()(address)" --rpc-url $RPC
# Must return: ANDE token address

# 4. Test proposal creation still works
# Create test proposal
# Verify it succeeds
```

#### Step 5.2: Test v2 Features

**Test 1: Rate Limiting**
```bash
# Create proposal 1
cast send $GOVERNOR "propose(...)" --private-key $USER_KEY

# Try to create proposal 2 immediately (should fail)
cast send $GOVERNOR "propose(...)" --private-key $USER_KEY
# Expected error: ProposalCooldownNotExpired

# Check cooldown time
cast call $GOVERNOR "lastProposalTime(address)(uint256)" $USER_ADDRESS --rpc-url $RPC

# Wait 1 day, try again (should succeed)
```

**Test 2: Anti-Whale**
```bash
# User with >10% supply votes
# Check event logs for VotingPowerCapped
# Verify actual vote weight is capped at 10%
```

**Test 3: Guardian Cancel**
```bash
# Create test proposal
PROPOSAL_ID=...

# Guardian cancels it (via multisig)
# 5/7 signers approve transaction:
cast send $GOVERNOR "guardianCancel(address[],uint256[],bytes[],bytes32,string)" \
  ... \
  --private-key $GUARDIAN_KEY

# Verify proposal cancelled
cast call $GOVERNOR "state(uint256)(uint8)" $PROPOSAL_ID --rpc-url $RPC
# Should return 2 (Canceled)
```

#### Step 5.3: Monitor for Issues

**Week 1 Post-Upgrade:**
- Watch all proposal creations
- Monitor guardian actions (should be 0)
- Check for any reverts
- Verify gas costs reasonable

**Week 2-4:**
- Process 5+ normal proposals
- Ensure quorum calculations work
- Verify staking bonus still applies
- Community feedback positive

---

## 🚨 Emergency Procedures

### If Upgrade Fails During Execution

1. **Check error message**
   ```bash
   cast receipt $TX_HASH --rpc-url $RPC
   ```

2. **Common issues:**
   - Storage collision → Verify storage layout
   - Init failed → Check guardian address valid
   - Gas limit → Increase gas limit
   - Timelock not elapsed → Wait longer

3. **If critical bug found after upgrade:**
   - Guardian pauses if possible
   - Create emergency proposal for v2.1 fix
   - Fast-track voting (EMERGENCY type)
   - Deploy fix within 24 hours

### Rollback to v1 (Last Resort)

```bash
# Deploy v1 implementation again
cast send --create $(jq -r '.bytecode.object' out/AndeGovernor.sol/AndeGovernor.json) \
  --rpc-url $RPC --private-key $DEPLOYER_KEY

# Create governance proposal to downgrade
# Vote, queue, execute
# System back to v1 state
```

---

## 📊 Cost Estimate

```
Deploy v2 Implementation:    ~5M gas
Create Proposal:             ~300K gas
Vote (100 users):            ~10M gas total
Queue:                       ~150K gas
Execute (upgrade):           ~500K gas
-------------------------------------------
Total:                       ~16M gas

At 2 gwei, $2000 ETH:        ~$64
At 10 gwei, $2000 ETH:       ~$320
```

---

## ✅ Pre-Upgrade Checklist

### Technical
- [ ] v2 implementation deployed and verified
- [ ] Storage layout compatibility confirmed
- [ ] Contract size < 24 KB
- [ ] All manual tests passing on fork
- [ ] Guardian multisig created and tested
- [ ] Upgrade calldata prepared and verified
- [ ] Monitoring alerts configured

### Operational  
- [ ] Team trained on v2 features
- [ ] Guardian signers briefed
- [ ] Emergency procedures documented
- [ ] Rollback plan ready

### Community
- [ ] Announcement published (T-2 weeks)
- [ ] Documentation updated
- [ ] Voting instructions clear
- [ ] Support channels ready

---

## 📝 Post-Upgrade Report Template

```markdown
# AndeGovernor v2 Upgrade Report

**Date:** YYYY-MM-DD
**Proposal ID:** #X
**Execution Block:** #XXXXX
**Execution TX:** 0x...

## Results
- [ ] Upgrade successful
- [ ] Version verified: 2.0.0
- [ ] Guardian set: 0x...
- [ ] All v1 features working
- [ ] All v2 features working
- [ ] Zero critical issues

## Metrics
- Voting participation: X%
- Votes FOR: X ANDE
- Votes AGAINST: X ANDE
- Quorum: X% (required: Y%)
- Execution gas: X

## Issues Encountered
- None / List issues

## Next Steps
- [ ] Monitor for 1 week
- [ ] Process test proposals
- [ ] Gather community feedback
- [ ] Plan v3 features
```

---

## 🎯 Success Criteria

Upgrade is successful if:

1. ✅ v2 implementation deployed (size < 24KB)
2. ✅ Governance proposal passed (>4% quorum)
3. ✅ Upgrade executed without errors
4. ✅ Version returns "2.0.0"
5. ✅ Guardian set correctly
6. ✅ All v1 features work identically
7. ✅ Rate limiting enforced
8. ✅ Anti-whale cap applied
9. ✅ Guardian can cancel proposals
10. ✅ Zero critical bugs in week 1

---

## 📚 Key Learnings from v1 Deployment

**What went well:**
- Manual deployment caught issues early
- Storage layout planning prevented bugs
- Optimizer settings worked correctly

**What to improve for v2:**
- Test guardian multisig before proposal
- More time for community education
- Better monitoring from day 1

---

## 🔗 References

- UUPS Pattern: https://eips.ethereum.org/EIPS/eip-1822
- OpenZeppelin Upgrades: https://docs.openzeppelin.com/upgrades-plugins/
- Storage Gaps: https://docs.openzeppelin.com/contracts/4.x/upgradeable#storage_gaps
- Governor: https://docs.openzeppelin.com/contracts/4.x/api/governance

---

**Document Version:** 1.0.0
**Last Updated:** 2025-01-24
**Owner:** Ande Labs Core Team
**Status:** Ready for Implementation