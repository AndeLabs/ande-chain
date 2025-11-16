// SPDX-License-Identifier: MIT
pragma solidity ^0.8.25;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {AccessControlUpgradeable} from "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import {GovernorUpgradeable} from "@openzeppelin/contracts-upgradeable/governance/GovernorUpgradeable.sol";
import {GovernorSettingsUpgradeable} from
    "@openzeppelin/contracts-upgradeable/governance/extensions/GovernorSettingsUpgradeable.sol";
import {GovernorCountingSimpleUpgradeable} from
    "@openzeppelin/contracts-upgradeable/governance/extensions/GovernorCountingSimpleUpgradeable.sol";
import {GovernorVotesUpgradeable} from
    "@openzeppelin/contracts-upgradeable/governance/extensions/GovernorVotesUpgradeable.sol";
import {GovernorTimelockControlUpgradeable} from
    "@openzeppelin/contracts-upgradeable/governance/extensions/GovernorTimelockControlUpgradeable.sol";
import {IVotes} from "@openzeppelin/contracts/governance/utils/IVotes.sol";
import {AndeTimelockController} from "./AndeTimelockController.sol";
import {GovernorDualTokenVoting, IAndeNativeStaking} from "./extensions/GovernorDualTokenVoting.sol";
// Temporarily disabled to reduce contract size - will be added in v2 upgrade
// import {GovernorAdaptiveQuorum} from "./extensions/GovernorAdaptiveQuorum.sol";
// import {GovernorMultiLevel} from "./extensions/GovernorMultiLevel.sol";
// import {GovernorSecurityExtensions} from "./extensions/GovernorSecurityExtensions.sol";

/**
 * @title AndeGovernor
 * @author Ande Labs & Gemini
 * @notice The main governance contract for AndeChain.
 * @dev This contract manages proposals and voting. It is controlled by ANDE token holders.
 * 
 * ENHANCED FEATURES (v1.0 MVP):
 * - Dual Token Voting: Combines base token votes + staking bonus
 * - Users get bonus voting power based on their staking (lock period multipliers)
 * - Anti-whale protection: max 500% bonus over base votes + max 10% voting power cap
 * - Fixed Quorum: 10% of total supply
 * - TimelockController for secure execution
 * - Upgradeable using UUPS pattern
 *
 * VOTING POWER FORMULA:
 * totalVotes = baseVotes (from ANDETokenDuality) + stakingBonus (from AndeNativeStaking)
 * Capped at 10% of total supply per voter
 *
 * V2 UPGRADE ROADMAP:
 * - Adaptive Quorum (adjusts 4-15% based on participation)
 * - Multi-Level Proposals (OPERATIONAL, PROTOCOL, CRITICAL, EMERGENCY)
 * - Security Extensions (anti-whale, rate limiting, guardian)
 */
contract AndeGovernor is
    Initializable,
    UUPSUpgradeable,
    AccessControlUpgradeable,
    GovernorUpgradeable,
    GovernorSettingsUpgradeable,
    GovernorCountingSimpleUpgradeable,
    GovernorDualTokenVoting,
    GovernorTimelockControlUpgradeable
{
    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    /**
     * @notice Initializes the Governor contract.
     * @param _token The address of the ANDE token (which implements IVotes).
     * @param _stakingContract The address of the AndeNativeStaking contract.
     * @param _timelock The address of the `AndeTimelockController`.
     * @param _votingPeriod The duration of the voting period in blocks.
     * @param _votingDelay The delay before a vote starts in blocks.
     * @param _proposalThreshold The minimum number of votes required to create a proposal.
     * @param _emergencyCouncil Reserved for v2 upgrade (currently unused).
     */
    function initialize(
        IVotes _token,
        IAndeNativeStaking _stakingContract,
        AndeTimelockController _timelock,
        uint32 _votingPeriod,
        uint48 _votingDelay,
        uint256 _proposalThreshold,
        address _emergencyCouncil // Reserved for v2
    ) public initializer {
        __Governor_init("AndeGovernor");
        __GovernorSettings_init(_votingDelay, _votingPeriod, _proposalThreshold);
        __GovernorDualTokenVoting_init(_token, _stakingContract);
        __GovernorTimelockControl_init(_timelock);
        __AccessControl_init();
        __UUPSUpgradeable_init();

        // Grant DEFAULT_ADMIN_ROLE to the timelock (for upgrades)
        _grantRole(DEFAULT_ADMIN_ROLE, address(_timelock));

        // Silence unused variable warning (will be used in v2)
        _emergencyCouncil;
    }

    // --- Overrides for UUPS ---

    function _authorizeUpgrade(address newImplementation) internal override onlyRole(DEFAULT_ADMIN_ROLE) {}

    // --- Required Overrides ---

    function votingDelay() public view override(GovernorUpgradeable, GovernorSettingsUpgradeable) returns (uint256) {
        return super.votingDelay();
    }

    function votingPeriod() public view override(GovernorUpgradeable, GovernorSettingsUpgradeable) returns (uint256) {
        return super.votingPeriod();
    }

    /**
     * @notice Fixed quorum at 10% of total supply
     * @dev In v2, this will be replaced with adaptive quorum (4-15% based on participation)
     */
    function quorum(uint256 timepoint) public view override returns (uint256) {
        return (token().getPastTotalSupply(timepoint) * 10) / 100; // 10% fixed quorum
    }

    function state(uint256 proposalId)
        public
        view
        override(GovernorUpgradeable, GovernorTimelockControlUpgradeable)
        returns (ProposalState)
    {
        return super.state(proposalId);
    }

    function proposalNeedsQueuing(uint256 proposalId)
        public
        view
        override(GovernorUpgradeable, GovernorTimelockControlUpgradeable)
        returns (bool)
    {
        return super.proposalNeedsQueuing(proposalId);
    }

    function proposalThreshold()
        public
        view
        override(GovernorUpgradeable, GovernorSettingsUpgradeable)
        returns (uint256)
    {
        return super.proposalThreshold();
    }

    function _queueOperations(
        uint256 proposalId,
        address[] memory targets,
        uint256[] memory values,
        bytes[] memory calldatas,
        bytes32 descriptionHash
    ) internal override(GovernorUpgradeable, GovernorTimelockControlUpgradeable) returns (uint48) {
        return super._queueOperations(proposalId, targets, values, calldatas, descriptionHash);
    }

    function _executeOperations(
        uint256 proposalId,
        address[] memory targets,
        uint256[] memory values,
        bytes[] memory calldatas,
        bytes32 descriptionHash
    ) internal override(GovernorUpgradeable, GovernorTimelockControlUpgradeable) {
        return super._executeOperations(proposalId, targets, values, calldatas, descriptionHash);
    }

    function _cancel(
        address[] memory targets,
        uint256[] memory values,
        bytes[] memory calldatas,
        bytes32 descriptionHash
    ) internal override(GovernorUpgradeable, GovernorTimelockControlUpgradeable) returns (uint256) {
        return super._cancel(targets, values, calldatas, descriptionHash);
    }

    function _executor()
        internal
        view
        override(GovernorUpgradeable, GovernorTimelockControlUpgradeable)
        returns (address)
    {
        return super._executor();
    }


    function supportsInterface(bytes4 interfaceId)
        public
        view
        override(GovernorUpgradeable, AccessControlUpgradeable)
        returns (bool)
    {
        return super.supportsInterface(interfaceId);
    }

    function _getVotes(
        address account,
        uint256 timepoint,
        bytes memory params
    ) internal view override(GovernorUpgradeable, GovernorDualTokenVoting) returns (uint256) {
        return GovernorDualTokenVoting._getVotes(account, timepoint, params);
    }
}
