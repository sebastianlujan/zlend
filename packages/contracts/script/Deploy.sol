// SPDX-License-Identifier: MIT
pragma solidity 0.8.34;

import {HonkVerifier} from 'contracts/Verifier.sol';
import {MockAavePool} from 'contracts/mocks/MockAavePool.sol';
import {OGBankContract} from 'contracts/OGBankContract.sol';
import {Script} from 'forge-std/Script.sol';
import {IERC20} from 'forge-std/interfaces/IERC20.sol';
import {MockERC20} from '../test/unit/helpers/MockERC20.sol';

contract Deploy is Script {
  struct DeploymentParams {
    address aavePool;
    address collateralToken;
    address borrowToken;
  }

  /// @notice Deployment parameters for each chain
  mapping(uint256 _chainId => DeploymentParams _params) internal _deploymentParams;

  function setUp() public {
    // Sepolia
    _deploymentParams[11_155_111] = DeploymentParams({
      aavePool: address(0), // TODO: Aave V3 Sepolia pool address
      collateralToken: address(0),
      borrowToken: address(0)
    });

    // Avalanche C-Chain (43114)
    _deploymentParams[43_114] = DeploymentParams({
      aavePool: 0x794a61358D6845594F94dc1DB02A252b5b4814aD,
      collateralToken: 0xB31f66AA3C1e785363F0875A1B74E27b85FD66c7, // TODO: replace with ZEC token address
      borrowToken: 0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E // USDC
    });

    // Avalanche Fuji (43113)
    _deploymentParams[43_113] = DeploymentParams({
      aavePool: 0x794a61358D6845594F94dc1DB02A252b5b4814aD, // TODO: verify Fuji pool
      collateralToken: 0xd00ae08403B9bbb9124bB305C09058E32C39A48c, // TODO: replace with ZEC Fuji
      borrowToken: 0x5425890298aed601595a70AB815c96711a31Bc65 // USDC Fuji
    });
  }

  function run() public {
    vm.startBroadcast();

    // 1. Always deploy verifier
    HonkVerifier _verifier = new HonkVerifier();

    // 2. For local anvil: deploy mock infrastructure
    DeploymentParams memory _params = _deploymentParams[block.chainid];

    if (block.chainid == 31_337) {
      MockERC20 _collateral = new MockERC20('Zcash', 'ZEC', 8);
      MockERC20 _borrow = new MockERC20('USD Coin', 'USDC', 6);
      MockAavePool _pool = new MockAavePool();
      _params = DeploymentParams(address(_pool), address(_collateral), address(_borrow));

      // Mint 10k tokens to the deployer
      _collateral.mint(msg.sender, 10_000 * 10 ** 8);
      _borrow.mint(msg.sender, 10_000 * 10 ** 6);
    }

    // 3. Deploy OGBankContract if pool is configured
    if (_params.aavePool != address(0)) {
      new OGBankContract(address(_verifier), _params.aavePool, _params.collateralToken, _params.borrowToken);
    }

    vm.stopBroadcast();
  }
}
