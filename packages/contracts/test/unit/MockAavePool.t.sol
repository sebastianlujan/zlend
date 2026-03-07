// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.34;

import {MockAavePool} from 'contracts/mocks/MockAavePool.sol';
import {Test} from 'forge-std/Test.sol';
import {IERC20} from 'forge-std/interfaces/IERC20.sol';

import {MockERC20} from './helpers/MockERC20.sol';

contract UnitMockAavePool is Test {
  MockAavePool internal _pool;
  MockERC20 internal _collateral;
  MockERC20 internal _borrowToken;

  address internal _user = makeAddr('user');
  uint256 internal _supplyAmount = 1000e18;
  uint256 internal _borrowAmount = 500e18;

  event MockSupply(address indexed _asset, uint256 _amount, address indexed _onBehalfOf);
  event MockBorrow(address indexed _asset, uint256 _amount, address indexed _onBehalfOf);
  event MockRepay(address indexed _asset, uint256 _amount, address indexed _onBehalfOf);
  event MockWithdraw(address indexed _asset, uint256 _amount, address indexed _to);

  function setUp() external {
    _pool = new MockAavePool();
    _collateral = new MockERC20('Collateral', 'COL', 18);
    _borrowToken = new MockERC20('Borrow', 'BOR', 18);

    // Mint tokens to user for supply
    _collateral.mint(_user, _supplyAmount);

    // Mint borrow tokens to the pool so it can lend
    _borrowToken.mint(address(_pool), _borrowAmount * 10);
  }

  /*///////////////////////////////////////////////////////////////
                              SUPPLY
  //////////////////////////////////////////////////////////////*/

  function test_Supply_TransfersTokensFromCaller() external {
    vm.startPrank(_user);
    _collateral.approve(address(_pool), _supplyAmount);
    _pool.supply(address(_collateral), _supplyAmount, _user, 0);
    vm.stopPrank();

    assertEq(_collateral.balanceOf(_user), 0);
    assertEq(_collateral.balanceOf(address(_pool)), _supplyAmount);
  }

  function test_Supply_TracksSuppliedAmount() external {
    vm.startPrank(_user);
    _collateral.approve(address(_pool), _supplyAmount);
    _pool.supply(address(_collateral), _supplyAmount, _user, 0);
    vm.stopPrank();

    assertEq(_pool.supplied(_user, address(_collateral)), _supplyAmount);
  }

  function test_Supply_EmitsEvent() external {
    vm.startPrank(_user);
    _collateral.approve(address(_pool), _supplyAmount);

    vm.expectEmit(true, true, true, true, address(_pool));
    emit MockSupply(address(_collateral), _supplyAmount, _user);

    _pool.supply(address(_collateral), _supplyAmount, _user, 0);
    vm.stopPrank();
  }

  function test_Supply_OnBehalfOfDifferentUser() external {
    address _recipient = makeAddr('recipient');

    vm.startPrank(_user);
    _collateral.approve(address(_pool), _supplyAmount);
    _pool.supply(address(_collateral), _supplyAmount, _recipient, 0);
    vm.stopPrank();

    assertEq(_pool.supplied(_recipient, address(_collateral)), _supplyAmount);
    assertEq(_pool.supplied(_user, address(_collateral)), 0);
  }

  /*///////////////////////////////////////////////////////////////
                              BORROW
  //////////////////////////////////////////////////////////////*/

  function test_Borrow_TransfersTokensToCaller() external {
    vm.prank(_user);
    _pool.borrow(address(_borrowToken), _borrowAmount, 2, 0, _user);

    assertEq(_borrowToken.balanceOf(_user), _borrowAmount);
  }

  function test_Borrow_TracksBorrowedAmount() external {
    vm.prank(_user);
    _pool.borrow(address(_borrowToken), _borrowAmount, 2, 0, _user);

    assertEq(_pool.borrowed(_user, address(_borrowToken)), _borrowAmount);
  }

  function test_Borrow_EmitsEvent() external {
    vm.expectEmit(true, true, true, true, address(_pool));
    emit MockBorrow(address(_borrowToken), _borrowAmount, _user);

    vm.prank(_user);
    _pool.borrow(address(_borrowToken), _borrowAmount, 2, 0, _user);
  }

  /*///////////////////////////////////////////////////////////////
                              REPAY
  //////////////////////////////////////////////////////////////*/

  function test_Repay_ReducesBorrowedAmount() external {
    // Setup: borrow first
    vm.prank(_user);
    _pool.borrow(address(_borrowToken), _borrowAmount, 2, 0, _user);

    // Repay
    vm.startPrank(_user);
    _borrowToken.approve(address(_pool), _borrowAmount);
    uint256 _repaid = _pool.repay(address(_borrowToken), _borrowAmount, 2, _user);
    vm.stopPrank();

    assertEq(_repaid, _borrowAmount);
    assertEq(_pool.borrowed(_user, address(_borrowToken)), 0);
  }

  function test_Repay_CapsAtDebtAmount() external {
    // Setup: borrow less than repay amount
    vm.prank(_user);
    _pool.borrow(address(_borrowToken), _borrowAmount, 2, 0, _user);

    uint256 _excessRepay = _borrowAmount * 2;

    vm.startPrank(_user);
    _borrowToken.approve(address(_pool), _excessRepay);
    uint256 _repaid = _pool.repay(address(_borrowToken), _excessRepay, 2, _user);
    vm.stopPrank();

    // Should only repay the actual debt
    assertEq(_repaid, _borrowAmount);
    assertEq(_pool.borrowed(_user, address(_borrowToken)), 0);
  }

  function test_Repay_EmitsEvent() external {
    vm.prank(_user);
    _pool.borrow(address(_borrowToken), _borrowAmount, 2, 0, _user);

    vm.startPrank(_user);
    _borrowToken.approve(address(_pool), _borrowAmount);

    vm.expectEmit(true, true, true, true, address(_pool));
    emit MockRepay(address(_borrowToken), _borrowAmount, _user);

    _pool.repay(address(_borrowToken), _borrowAmount, 2, _user);
    vm.stopPrank();
  }

  /*///////////////////////////////////////////////////////////////
                            WITHDRAW
  //////////////////////////////////////////////////////////////*/

  function test_Withdraw_TransfersTokensBack() external {
    // Setup: supply first
    vm.startPrank(_user);
    _collateral.approve(address(_pool), _supplyAmount);
    _pool.supply(address(_collateral), _supplyAmount, _user, 0);

    // Withdraw
    uint256 _withdrawn = _pool.withdraw(address(_collateral), _supplyAmount, _user);
    vm.stopPrank();

    assertEq(_withdrawn, _supplyAmount);
    assertEq(_collateral.balanceOf(_user), _supplyAmount);
    assertEq(_pool.supplied(_user, address(_collateral)), 0);
  }

  function test_Withdraw_CapsAtSuppliedAmount() external {
    vm.startPrank(_user);
    _collateral.approve(address(_pool), _supplyAmount);
    _pool.supply(address(_collateral), _supplyAmount, _user, 0);

    uint256 _excessWithdraw = _supplyAmount * 2;
    uint256 _withdrawn = _pool.withdraw(address(_collateral), _excessWithdraw, _user);
    vm.stopPrank();

    assertEq(_withdrawn, _supplyAmount);
  }

  function test_Withdraw_EmitsEvent() external {
    vm.startPrank(_user);
    _collateral.approve(address(_pool), _supplyAmount);
    _pool.supply(address(_collateral), _supplyAmount, _user, 0);

    vm.expectEmit(true, true, true, true, address(_pool));
    emit MockWithdraw(address(_collateral), _supplyAmount, _user);

    _pool.withdraw(address(_collateral), _supplyAmount, _user);
    vm.stopPrank();
  }

  function test_Withdraw_ToDifferentAddress() external {
    address _recipient = makeAddr('recipient');

    vm.startPrank(_user);
    _collateral.approve(address(_pool), _supplyAmount);
    _pool.supply(address(_collateral), _supplyAmount, _user, 0);

    _pool.withdraw(address(_collateral), _supplyAmount, _recipient);
    vm.stopPrank();

    assertEq(_collateral.balanceOf(_recipient), _supplyAmount);
  }
}
