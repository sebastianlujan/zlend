// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";

/// @title MockUSDC
/// @notice Simple ERC-20 for testing. Minted to deployer on construction.
contract MockUSDC is ERC20 {
    constructor(uint256 initialSupply) ERC20("Mock USDC", "mUSDC") {
        _mint(msg.sender, initialSupply);
    }

    function decimals() public pure override returns (uint8) {
        return 6;
    }
}
