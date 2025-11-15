// SPDX-License-Identifier: MIT
pragma solidity ^0.8.25;

import {Script, console2} from "forge-std/Script.sol";

/**
 * @title AuditDeployments
 * @notice Script para auditar el estado actual de contratos desplegados en la red
 * @dev Conecta con la red y verifica qué contratos están realmente desplegados
 * 
 * Usage:
 *   forge script script/AuditDeployments.s.sol --rpc-url $TESTNET_RPC
 */
contract AuditDeployments is Script {
    
    // Direcciones conocidas a verificar (actualizar según sea necesario)
    address[] public knownAddresses = [
        0x5FC8d32690cc91D4c39d9d3abcBD16989F875707, // ANDE Token Proxy
        0xa513E6E4b8f2a923D98304ec87F64353C4D5C853, // Staking Proxy
        0x8A791620dd6260079BF849Dc5567aDC3F2FdC318, // Timelock Proxy
        0xB7f8BC63BbcaD18155201308C8f3540b07f84F5e  // Governor Proxy
    ];
    
    string[] public contractNames = [
        "ANDE Token Proxy",
        "AndeNativeStaking Proxy",
        "AndeTimelockController Proxy",
        "AndeGovernor Proxy"
    ];
    
    function run() external view {
        console2.log("==========================================================");
        console2.log("          ANDECHAIN DEPLOYMENT AUDIT");
        console2.log("==========================================================");
        console2.log("Chain ID:", block.chainid);
        console2.log("Block Number:", block.number);
        console2.log("Timestamp:", block.timestamp);
        console2.log("");
        
        console2.log("=== Auditing Known Contract Addresses ===");
        console2.log("");
        
        for (uint256 i = 0; i < knownAddresses.length; i++) {
            _auditContract(knownAddresses[i], contractNames[i]);
        }
        
        console2.log("==========================================================");
        console2.log("Audit Complete");
        console2.log("==========================================================");
    }
    
    function _auditContract(address contractAddress, string memory name) internal view {
        uint256 codeSize;
        assembly {
            codeSize := extcodesize(contractAddress)
        }
        
        console2.log("Contract:", name);
        console2.log("  Address:", contractAddress);
        console2.log("  Code Size:", codeSize, "bytes");
        console2.log("  Status:", codeSize > 0 ? "DEPLOYED" : "NOT DEPLOYED");
        
        if (codeSize > 0) {
            console2.log("  Balance:", contractAddress.balance / 1 ether, "ANDE");
        }
        
        console2.log("");
    }
}
