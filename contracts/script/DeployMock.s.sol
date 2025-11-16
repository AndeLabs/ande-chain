// SPDX-License-Identifier: MIT
pragma solidity 0.8.25;

import {Script, console2} from "forge-std/Script.sol";
import {NativeTransferPrecompileMock} from "../src/mocks/NativeTransferPrecompileMock.sol";

/**
 * @title DeployMock
 * @notice Deploys the NativeTransferPrecompileMock for Token Duality testing
 */
contract DeployMock is Script {
    function run() external {
        // ANDE Token Proxy address from testnet-6174-production.json
        address andeProxy = 0x5FC8d32690cc91D4c39d9d3abcBD16989F875707;

        console2.log("=========================================================");
        console2.log("      Deploying NativeTransferPrecompileMock");
        console2.log("=========================================================");
        console2.log("ANDE Proxy (authorized caller):", andeProxy);
        console2.log("");

        vm.startBroadcast(vm.envUint("PRIVATE_KEY"));

        // Deploy the mock with ANDE proxy as authorized caller
        NativeTransferPrecompileMock mock = new NativeTransferPrecompileMock(andeProxy);

        vm.stopBroadcast();

        console2.log("=========================================================");
        console2.log("                 DEPLOYMENT SUCCESSFUL");
        console2.log("=========================================================");
        console2.log("NativeTransferPrecompileMock:", address(mock));
        console2.log("");
        console2.log("Next Steps:");
        console2.log("1. Update ANDE token to use this mock address");
        console2.log("   cast send", andeProxy, "\"setPrecompileAddress(address)\"", address(mock));
        console2.log("2. Then mint and test balances");
        console2.log("=========================================================");
    }
}
