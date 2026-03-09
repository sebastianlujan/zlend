// SPDX-License-Identifier: MIT
pragma solidity 0.8.34;

import {OGBankContract} from 'contracts/OGBankContract.sol';
import {Script} from 'forge-std/Script.sol';
import {MockERC20} from '../test/unit/helpers/MockERC20.sol';

/**
 * @title FundProtocol
 * @notice Funds the OGBank protocol on local anvil:
 *   1. Mints WAVAX to deployer and supplies as protocol collateral to Aave
 *   2. Mints USDT to MockAavePool for borrow liquidity
 *   3. Mints USDT to deployer for testing repay flow
 * @dev Reads deployed addresses from environment variables set by deploy-and-env.sh
 */
contract FundProtocol is Script {
  function run() public {
    address _ogBank = vm.envAddress('OGBANK_ADDRESS');
    address _collateralToken = vm.envAddress('COLLATERAL_TOKEN_ADDRESS');
    address _borrowToken = vm.envAddress('BORROW_TOKEN_ADDRESS');
    address _aavePool = vm.envAddress('AAVE_POOL_ADDRESS');

    vm.startBroadcast();

    // 1. Mint WAVAX to deployer and supply as protocol collateral
    uint256 _collateralAmount = 100 * 10 ** 18; // 100 WAVAX
    MockERC20(_collateralToken).mint(msg.sender, _collateralAmount);
    MockERC20(_collateralToken).approve(_ogBank, _collateralAmount);
    OGBankContract(_ogBank).supplyCollateral(_collateralAmount);

    // 2. Mint USDT to Aave pool for borrow liquidity
    MockERC20(_borrowToken).mint(_aavePool, 50_000 * 10 ** 6);

    // 3. Mint USDT to deployer for testing repay flow
    MockERC20(_borrowToken).mint(msg.sender, 10_000 * 10 ** 6);

    vm.stopBroadcast();
  }
}
