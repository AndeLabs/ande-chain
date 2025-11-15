# Account Abstraction (ERC-4337) for AndeChain

Complete implementation of [ERC-4337 Account Abstraction](https://eips.ethereum.org/EIPS/eip-4337) for AndeChain, enabling gasless transactions paid with ANDE tokens.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Components](#components)
- [Deployment](#deployment)
- [Usage Examples](#usage-examples)
- [Integration Guide](#integration-guide)
- [Security Considerations](#security-considerations)

## Overview

Account Abstraction allows users to interact with AndeChain using **smart contract wallets** instead of EOAs (Externally Owned Accounts). Key benefits:

- ✅ **Gasless Transactions**: Pay gas fees with ANDE tokens instead of native ETH
- ✅ **Batch Operations**: Execute multiple transactions atomically
- ✅ **Social Recovery**: Recover accounts without seed phrases
- ✅ **Custom Validation**: Implement any signature scheme (multisig, passkeys, etc.)
- ✅ **Sponsored Gas**: Allow dApps to sponsor user gas fees

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        User / dApp                           │
└────────────────────┬────────────────────────────────────────┘
                     │
                     │ UserOperation
                     ▼
           ┌─────────────────────┐
           │    Bundler (Relayer) │
           └──────────┬───────────┘
                      │ handleOps()
                      ▼
    ╔═════════════════════════════════════════════════════════╗
    ║                     EntryPoint                           ║
    ║  (Canonical ERC-4337 v0.6 - 0x5FF137D4...)             ║
    ╚══════════════╤═════════════════════════╤════════════════╝
                   │                         │
        ┌──────────▼─────────┐    ┌─────────▼──────────┐
        │  SimpleAccount     │    │  ANDEPaymaster     │
        │  (Smart Wallet)    │    │  (Gas Sponsor)     │
        └────────────────────┘    └──────────┬─────────┘
                                              │
                                    ┌─────────▼──────────┐
                                    │  PriceOracle       │
                                    │  (ANDE/ETH rate)   │
                                    └────────────────────┘
```

### Flow Diagram

1. **User** creates a `UserOperation` (like a transaction, but richer)
2. **Bundler** receives UserOps from multiple users
3. **EntryPoint** validates and executes UserOps atomically
4. **Account** validates signature and executes the user's intent
5. **Paymaster** (optional) sponsors gas using ANDE tokens

## Components

### Core Contracts

#### EntryPoint.sol
**Official ERC-4337 v0.6 implementation** (630 lines, battle-tested)

- Handles all UserOperation execution
- Validates signatures and gas payments
- Manages deposits for accounts and paymasters
- Canonical address: `0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789`

```solidity
interface IEntryPoint {
    function handleOps(
        UserOperation[] calldata ops,
        address payable beneficiary
    ) external;

    function depositTo(address account) external payable;
    function balanceOf(address account) external view returns (uint256);
}
```

#### SimpleAccount.sol
**Smart contract wallet with single-signer validation**

- ECDSA signature validation
- Upgradeable via UUPS proxy
- Execute single or batch transactions
- Counterfactual deployment (CREATE2)

```solidity
interface IAccount {
    function execute(address dest, uint256 value, bytes calldata func) external;
    function executeBatch(address[] calldata dest, bytes[] calldata func) external;
}
```

#### SimpleAccountFactory.sol
**Deterministic account deployment factory**

- Uses CREATE2 for predictable addresses
- Deploys minimal proxies (ERC1967)
- Gas-efficient account creation

```solidity
interface IAccountFactory {
    function createAccount(address owner, uint256 salt) external returns (SimpleAccount);
    function getAddress(address owner, uint256 salt) external view returns (address);
}
```

### AndeChain-Specific Contracts

#### ANDEPaymaster.sol
**Custom paymaster accepting ANDE tokens for gas**

Features:
- **External Token**: Uses existing ANDE token (not self-minted)
- **Dynamic Pricing**: Integrates with PriceOracle for live ANDE/ETH rates
- **Whitelist Support**: Optional sponsored accounts
- **Gas Limits**: Configurable maximum gas per UserOp
- **Price Caching**: 60-second cache to reduce oracle calls

```solidity
interface IANDEPaymaster {
    // View functions
    function getANDEToken() external view returns (address);
    function getPriceOracle() external view returns (address);
    function getCurrentExchangeRate() external view returns (uint256);
    function calculateANDECost(uint256 gasUsed, uint256 gasPrice)
        external view returns (uint256);

    // Owner functions
    function setPriceOracle(IPriceOracle newOracle) external;
    function setMaxGasLimit(uint256 newLimit) external;
    function addToWhitelist(address account) external;
}
```

#### IPriceOracle.sol
**Interface for ANDE/ETH price feeds**

```solidity
interface IPriceOracle {
    function getMedianPrice(address token) external view returns (uint256);
}
```

## Deployment

### Prerequisites

1. **ANDE Token** must be deployed first
2. **RPC endpoint** for AndeChain (local or testnet)
3. **Deployer account** with ETH for gas

### Environment Setup

Create `.env` file:

```bash
PRIVATE_KEY=0x...
DEPLOYER_ADDRESS=0x...
ANDE_TOKEN_ADDRESS=0x5FbDB2315678afecb367f032d93F642f64180aa3
RPC_URL=http://localhost:8545
```

### Deploy Script

```bash
cd contracts

# Deploy Account Abstraction infrastructure
forge script script/DeployAccountAbstraction.s.sol \
    --rpc-url $RPC_URL \
    --broadcast \
    --legacy

# Addresses saved to: deployments/account-abstraction.env
```

### Deployment Output

```
EntryPoint:            0x522B3294E6d06aA25Ad0f1B8891242E335D3B459
ANDE Token:            0x535B3D7A252fa034Ed71F0C53ec0C6F784cB64E1
Price Oracle:          0xc051134F56d56160E8c8ed9bB3c439c78AB27cCc
SimpleAccountFactory:  0x2c1DE3b4Dbb4aDebEbB5dcECAe825bE2a9fc6eb6
ANDEPaymaster:         0x83769BeEB7e5405ef0B7dc3C66C43E3a51A6d27f
```

## Usage Examples

### 1. Create a Smart Wallet

```solidity
import {SimpleAccountFactory} from "./account/factories/SimpleAccountFactory.sol";

// Deploy counterfactual account
address owner = 0x...;  // EOA that controls the account
uint256 salt = 0;       // Unique salt for deterministic address

SimpleAccount account = factory.createAccount(owner, salt);

// Get address without deploying (for receiving funds)
address predictedAddress = factory.getAddress(owner, salt);
```

### 2. Construct a UserOperation

```typescript
import { ethers } from 'ethers';

const userOp = {
    sender: account.address,
    nonce: await entryPoint.getNonce(account.address, 0),
    initCode: '0x', // Empty if account already deployed
    callData: account.interface.encodeFunctionData('execute', [
        targetContract,
        ethers.parseEther('0'),
        targetContract.interface.encodeFunctionData('transfer', [recipient, amount])
    ]),
    callGasLimit: 100000,
    verificationGasLimit: 200000,
    preVerificationGas: 21000,
    maxFeePerGas: await provider.getGasPrice(),
    maxPriorityFeePerGas: ethers.parseUnits('1', 'gwei'),
    paymasterAndData: ethers.concat([
        andePaymaster.address,
        '0x1234' // Optional paymaster-specific data
    ]),
    signature: '0x' // Will be filled after signing
};
```

### 3. Sign UserOperation

```typescript
// Hash the UserOperation
const userOpHash = await entryPoint.getUserOpHash(userOp);

// Sign with account owner's private key
const signature = await owner.signMessage(ethers.getBytes(userOpHash));

userOp.signature = signature;
```

### 4. Pay Gas with ANDE Tokens

```typescript
// Approve ANDEPaymaster to spend ANDE from the smart wallet
await andeToken.connect(accountOwner).approve(
    andePaymaster.address,
    ethers.parseEther('100') // Approve 100 ANDE
);

// Check estimated ANDE cost
const gasCost = 200000; // Total gas estimate
const gasPrice = await provider.getGasPrice();
const andeCost = await andePaymaster.calculateANDECost(gasCost, gasPrice);

console.log(`Estimated gas cost: ${ethers.formatEther(andeCost)} ANDE`);
```

### 5. Submit to Bundler

```typescript
// Send UserOperation to bundler (off-chain)
const bundlerRPC = 'https://bundler.andechain.io';

const response = await fetch(bundlerRPC, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
        jsonrpc: '2.0',
        method: 'eth_sendUserOperation',
        params: [userOp, entryPoint.address],
        id: 1
    })
});

const { result: userOpHash } = await response.json();
console.log('UserOperation submitted:', userOpHash);
```

### 6. Batch Transactions

```typescript
// Execute multiple operations atomically
const targets = [tokenA, tokenB, nftContract];
const callDatas = [
    tokenA.interface.encodeFunctionData('transfer', [recipient1, amount1]),
    tokenB.interface.encodeFunctionData('approve', [spender, amount2]),
    nftContract.interface.encodeFunctionData('mint', [recipient2])
];

const callData = account.interface.encodeFunctionData('executeBatch', [
    targets,
    callDatas
]);

// Include in UserOperation's callData field
userOp.callData = callData;
```

## Integration Guide

### For dApp Developers

#### Option A: Use SDK (Recommended)

```bash
npm install @alchemy/aa-sdk
# or
npm install userop
```

```typescript
import { SimpleSmartAccount, createUserOp } from '@alchemy/aa-sdk';

const provider = new SimpleSmartAccountProvider(
    rpcUrl,
    entryPointAddress,
    chainId
);

// Send transaction with ANDE gas payment
const txHash = await provider.sendUserOperation({
    target: tokenContract,
    data: tokenContract.interface.encodeFunctionData('transfer', [to, amount]),
    paymaster: andePaymasterAddress
});
```

#### Option B: Build Custom Integration

See `contracts/test/account/ANDEPaymaster.t.sol` for Solidity examples.

### For Bundler Operators

Run an ERC-4337 bundler to relay UserOperations:

```bash
# Using Infinitism bundler
git clone https://github.com/eth-infinitism/bundler
cd bundler
yarn && yarn preprocess
yarn hardhat-deploy --network andechain
yarn run bundler --network andechain
```

Configuration (`bundler.config.json`):
```json
{
    "network": "andechain",
    "entryPoint": "0x522B3294E6d06aA25Ad0f1B8891242E335D3B459",
    "beneficiary": "0x...",
    "minBalance": "1000000000000000000",
    "mnemonic": "test test test..."
}
```

### For Wallet Providers

Implement ERC-4337 support:

1. **Account Creation**: Use `SimpleAccountFactory.createAccount()`
2. **UserOp Construction**: Build and sign UserOperations
3. **Paymaster Integration**: Add ANDE token support
4. **Bundler Connection**: Submit UserOps to bundler RPC

See [ERC-4337 Wallet Guide](https://docs.alchemy.com/docs/account-abstraction-wallet-guide).

## Security Considerations

### Paymaster Security

✅ **Implemented Protections**:
- Whitelist for sponsored accounts
- Maximum gas limits per UserOp
- Validates account factory during creation
- Price cache to prevent oracle manipulation
- SafeERC20 for token transfers

⚠️ **Operator Responsibilities**:
- Monitor paymaster ETH balance in EntryPoint
- Ensure price oracle is reliable and tamper-proof
- Regularly update whitelist
- Set appropriate gas limits

### Account Security

✅ **Implemented Protections**:
- Signature validation (ECDSA)
- Owner-only upgrade authorization
- Nonce management prevents replay attacks

⚠️ **User Responsibilities**:
- Protect private key of account owner
- Verify contract addresses before sending funds
- Use hardware wallets for high-value accounts

### General Best Practices

- **Audit Contracts**: EntryPoint uses canonical audited code
- **Test Thoroughly**: See `test/account/` for comprehensive tests
- **Monitor Gas Prices**: Adjust ANDE price oracle regularly
- **Limit Paymaster Exposure**: Fund incrementally, monitor usage
- **Use Multisig**: For paymaster owner and oracle admin

## Testing

Run comprehensive test suite:

```bash
# Unit tests
forge test --match-path test/account/ANDEPaymaster.t.sol

# Integration tests
forge test --match-path test/account/EntryPoint.t.sol

# With gas reporting
forge test --gas-report --match-contract ANDEPaymaster

# Coverage
forge coverage --report lcov
```

## References

- [EIP-4337: Account Abstraction](https://eips.ethereum.org/EIPS/eip-4337)
- [Official ERC-4337 Contracts](https://github.com/eth-infinitism/account-abstraction)
- [Alchemy AA SDK](https://accountkit.alchemy.com/)
- [Stackup Bundler](https://docs.stackup.sh/docs)

## License

GPL-3.0 (matches official ERC-4337 contracts)

---

**Questions?** Open an issue on GitHub or reach out to the Ande Labs team.
