// SPDX-License-Identifier: MIT
pragma solidity 0.8.34;

import {IAavePool} from 'interfaces/IAavePool.sol';
import {IERC20} from 'forge-std/interfaces/IERC20.sol';

/**
 * @title MockAavePool
 * @notice Simplified mock of Aave V3 Pool for unit testing
 * @dev Tracks supply/borrow balances per user per asset and performs actual ERC20 transfers.
 *   Does NOT implement interest accrual, liquidation, or health factor checks.
 */
contract MockAavePool is IAavePool {
  /*///////////////////////////////////////////////////////////////
                              EVENTS
  //////////////////////////////////////////////////////////////*/

  event MockSupply(address indexed _asset, uint256 _amount, address indexed _onBehalfOf);
  event MockBorrow(address indexed _asset, uint256 _amount, address indexed _onBehalfOf);
  event MockRepay(address indexed _asset, uint256 _amount, address indexed _onBehalfOf);
  event MockWithdraw(address indexed _asset, uint256 _amount, address indexed _to);

  /*///////////////////////////////////////////////////////////////
                              STATE
  //////////////////////////////////////////////////////////////*/

  /// @notice Tracks supplied amounts: user => asset => amount
  mapping(address => mapping(address => uint256)) public supplied;

  /// @notice Tracks borrowed amounts: user => asset => amount
  mapping(address => mapping(address => uint256)) public borrowed;

  /*///////////////////////////////////////////////////////////////
                              LOGIC
  //////////////////////////////////////////////////////////////*/

  /// @inheritdoc IAavePool
  function supply(address _asset, uint256 _amount, address _onBehalfOf, uint16) external {
    IERC20(_asset).transferFrom(msg.sender, address(this), _amount);
    supplied[_onBehalfOf][_asset] += _amount;
    emit MockSupply(_asset, _amount, _onBehalfOf);
  }

  /// @inheritdoc IAavePool
  function borrow(address _asset, uint256 _amount, uint256, uint16, address _onBehalfOf) external {
    borrowed[_onBehalfOf][_asset] += _amount;
    IERC20(_asset).transfer(msg.sender, _amount);
    emit MockBorrow(_asset, _amount, _onBehalfOf);
  }

  /// @inheritdoc IAavePool
  function repay(address _asset, uint256 _amount, uint256, address _onBehalfOf) external returns (uint256) {
    uint256 _debt = borrowed[_onBehalfOf][_asset];
    uint256 _actual = _amount > _debt ? _debt : _amount;
    IERC20(_asset).transferFrom(msg.sender, address(this), _actual);
    borrowed[_onBehalfOf][_asset] -= _actual;
    emit MockRepay(_asset, _actual, _onBehalfOf);
    return _actual;
  }

  /// @inheritdoc IAavePool
  function withdraw(address _asset, uint256 _amount, address _to) external returns (uint256) {
    uint256 _balance = supplied[msg.sender][_asset];
    uint256 _actual = _amount > _balance ? _balance : _amount;
    supplied[msg.sender][_asset] -= _actual;
    IERC20(_asset).transfer(_to, _actual);
    emit MockWithdraw(_asset, _actual, _to);
    return _actual;
  }
}
