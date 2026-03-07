// SPDX-License-Identifier: MIT
pragma solidity 0.8.34;

import {OGBankContract} from 'contracts/OGBankContract.sol';
import {Script} from 'forge-std/Script.sol';
import {IERC20} from 'forge-std/interfaces/IERC20.sol';

contract Deploy is Script {
  struct DeploymentParams {
    IERC20 token;
    address verifier;
    address aavePool;
    address collateralToken;
    address borrowToken;
  }

  /// @notice Deployment parameters for each chain
  mapping(uint256 _chainId => DeploymentParams _params) internal _deploymentParams;

  function setUp() public {
    // Mainnet
    _deploymentParams[1] = DeploymentParams({
      token: IERC20(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2),
      verifier: address(0), // TODO: deploy verifier first
      aavePool: address(0), // No Aave V3 on Ethereum mainnet for OGBank
      collateralToken: address(0),
      borrowToken: address(0)
    });

    // Sepolia
    _deploymentParams[11_155_111] = DeploymentParams({
      token: IERC20(0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14),
      verifier: address(0), // TODO: deploy verifier first
      aavePool: address(0), // TODO: Aave V3 Sepolia pool address
      collateralToken: address(0),
      borrowToken: address(0)
    });

    // Avalanche C-Chain (43114)
    _deploymentParams[43_114] = DeploymentParams({
      token: IERC20(0xB31f66AA3C1e785363F0875A1B74E27b85FD66c7), // WAVAX
      verifier: address(0), // TODO: deploy verifier first
      aavePool: 0x794a61358D6845594F94dc1DB02A252b5b4814aD, // Aave V3 Pool on Avalanche
      collateralToken: 0xB31f66AA3C1e785363F0875A1B74E27b85FD66c7, // WAVAX
      borrowToken: 0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E // USDC
    });
  }

  function run() public {
    DeploymentParams memory _params = _deploymentParams[block.chainid];

    vm.startBroadcast();

    // Deploy OGBankContract if Aave pool is configured
    if (_params.aavePool != address(0) && _params.verifier != address(0)) {
      new OGBankContract(_params.verifier, _params.aavePool, _params.collateralToken, _params.borrowToken);
    }
    vm.stopBroadcast();
  }
}
