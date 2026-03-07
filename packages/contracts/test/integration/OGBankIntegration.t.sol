// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.34;

import {IOGBankContract} from 'interfaces/IOGBankContract.sol';
import {MockAavePool} from 'contracts/mocks/MockAavePool.sol';
import {MockVerifier} from 'contracts/mocks/MockVerifier.sol';
import {OGBankContract} from 'contracts/OGBankContract.sol';
import {Test} from 'forge-std/Test.sol';

import {MockERC20} from '../unit/helpers/MockERC20.sol';

contract IntegrationOGBank is Test {
  OGBankContract internal _ogBank;
  MockAavePool internal _aavePool;
  MockVerifier internal _mockVerifier;
  MockERC20 internal _collateral;
  MockERC20 internal _borrowToken;

  address internal _user = makeAddr('user');
  address internal _user2 = makeAddr('user2');
  uint256 internal _supplyAmount = 1000e18;
  uint256 internal _borrowAmount = 500e18;
  bytes32 internal _borrowNf = keccak256('integration_borrow_1');

  event CollateralSupplied(address indexed _user, uint256 _amount);
  event BorrowExecuted(address indexed _user, uint256 _amount, bytes32 indexed _borrowNullifier);
  event RepayExecuted(address indexed _user, uint256 _amount, bytes32 indexed _borrowNullifier);
  event FinishPayment(address indexed _ogbank, uint256 _amount, address _recipient, address _originAddress);

  function setUp() external {
    _mockVerifier = new MockVerifier();
    _aavePool = new MockAavePool();
    _collateral = new MockERC20('Collateral', 'COL', 18);
    _borrowToken = new MockERC20('BorrowToken', 'BOR', 18);

    _ogBank = new OGBankContract(
      address(_mockVerifier),
      address(_aavePool),
      address(_collateral),
      address(_borrowToken)
    );

    // Fund user with collateral
    _collateral.mint(_user, _supplyAmount);
    // Fund user2 with collateral
    _collateral.mint(_user2, _supplyAmount);
    // Fund Aave pool with borrow tokens
    _borrowToken.mint(address(_aavePool), _borrowAmount * 20);
  }

  /*///////////////////////////////////////////////////////////////
                    A. FULL LIFECYCLE WITH BALANCES
  //////////////////////////////////////////////////////////////*/

  function test_FullCycle_BalancesAtEveryStep() external {
    // -- Pre-state --
    assertEq(_collateral.balanceOf(_user), _supplyAmount);
    assertEq(_borrowToken.balanceOf(_user), 0);

    // 1. Supply collateral
    _supplyCollateral(_user, _supplyAmount);

    assertEq(_collateral.balanceOf(_user), 0, 'user collateral after supply');
    assertEq(_collateral.balanceOf(address(_aavePool)), _supplyAmount, 'aave collateral after supply');
    assertEq(_aavePool.supplied(address(_ogBank), address(_collateral)), _supplyAmount, 'aave tracked supply');

    // 2. Borrow
    _executeBorrow(_user, _borrowAmount, _borrowNf);

    assertEq(_borrowToken.balanceOf(_user), _borrowAmount, 'user borrow tokens after borrow');
    assertEq(_aavePool.borrowed(address(_ogBank), address(_borrowToken)), _borrowAmount, 'aave tracked borrow');

    // 3. Repay
    _executeRepay(_user, _borrowAmount, _borrowNf);

    assertEq(_borrowToken.balanceOf(_user), 0, 'user borrow tokens after repay');
    assertEq(_aavePool.borrowed(address(_ogBank), address(_borrowToken)), 0, 'aave debt after repay');

    // 4. Withdraw
    _executeWithdraw(_user, _supplyAmount, _borrowNf);

    assertEq(_collateral.balanceOf(_user), _supplyAmount, 'user collateral after withdraw');
  }

  function test_FullCycle_NullifierChainState() external {
    _supplyCollateral(_user, _supplyAmount);

    // After borrow
    _executeBorrow(_user, _borrowAmount, _borrowNf);
    assertTrue(_ogBank.borrowNullifiers(_borrowNf), 'borrow nullifier set');
    assertFalse(_ogBank.consumedNullifiers(_borrowNf), 'not consumed yet');

    // After repay
    _executeRepay(_user, _borrowAmount, _borrowNf);
    bytes32 _repayNf = keccak256(abi.encodePacked(_borrowNf, _user));
    assertTrue(_ogBank.repayNullifiers(_repayNf), 'repay nullifier set');
    assertFalse(_ogBank.consumedNullifiers(_borrowNf), 'still not consumed');

    // After withdraw
    _executeWithdraw(_user, _supplyAmount, _borrowNf);
    assertTrue(_ogBank.consumedNullifiers(_borrowNf), 'consumed after withdraw');
  }

  function test_FullCycle_EmitsAllEvents() external {
    // Supply event
    vm.startPrank(_user);
    _collateral.approve(address(_ogBank), _supplyAmount);
    vm.expectEmit(true, true, true, true, address(_ogBank));
    emit CollateralSupplied(_user, _supplyAmount);
    _ogBank.supplyCollateral(_supplyAmount);
    vm.stopPrank();

    // Borrow event
    bytes32[] memory _pubInputs = new bytes32[](1);
    _pubInputs[0] = _borrowNf;
    vm.expectEmit(true, true, true, true, address(_ogBank));
    emit BorrowExecuted(_user, _borrowAmount, _borrowNf);
    vm.prank(_user);
    _ogBank.borrow(hex'cafe', _pubInputs, _borrowAmount);

    // Repay event
    vm.startPrank(_user);
    _borrowToken.approve(address(_ogBank), _borrowAmount);
    vm.expectEmit(true, true, true, true, address(_ogBank));
    emit RepayExecuted(_user, _borrowAmount, _borrowNf);
    _ogBank.repay(_borrowAmount, _borrowNf);
    vm.stopPrank();

    // Withdraw event
    vm.expectEmit(true, true, true, true, address(_ogBank));
    emit FinishPayment(address(_ogBank), _supplyAmount, _user, _user);
    vm.prank(_user);
    _ogBank.withdrawProof(hex'cafe', _pubInputs, _supplyAmount);
  }

  /*///////////////////////////////////////////////////////////////
                      B. BALANCE ACCOUNTING
  //////////////////////////////////////////////////////////////*/

  function test_Balances_SupplyReducesUserCollateral() external {
    uint256 _userBefore = _collateral.balanceOf(_user);
    uint256 _aaveBefore = _collateral.balanceOf(address(_aavePool));

    _supplyCollateral(_user, _supplyAmount);

    assertEq(_collateral.balanceOf(_user), _userBefore - _supplyAmount);
    assertEq(_collateral.balanceOf(address(_aavePool)), _aaveBefore + _supplyAmount);
  }

  function test_Balances_BorrowGivesUserTokens() external {
    _supplyCollateral(_user, _supplyAmount);

    uint256 _userBorrowBefore = _borrowToken.balanceOf(_user);
    _executeBorrow(_user, _borrowAmount, _borrowNf);

    assertEq(_borrowToken.balanceOf(_user), _userBorrowBefore + _borrowAmount);
    assertEq(_aavePool.borrowed(address(_ogBank), address(_borrowToken)), _borrowAmount);
  }

  function test_Balances_RepayPullsUserTokens() external {
    _supplyCollateral(_user, _supplyAmount);
    _executeBorrow(_user, _borrowAmount, _borrowNf);

    uint256 _userBorrowBefore = _borrowToken.balanceOf(_user);
    _executeRepay(_user, _borrowAmount, _borrowNf);

    assertEq(_borrowToken.balanceOf(_user), _userBorrowBefore - _borrowAmount);
    assertEq(_aavePool.borrowed(address(_ogBank), address(_borrowToken)), 0);
  }

  function test_Balances_WithdrawReturnsCollateral() external {
    _supplyCollateral(_user, _supplyAmount);
    _executeBorrow(_user, _borrowAmount, _borrowNf);
    _executeRepay(_user, _borrowAmount, _borrowNf);

    uint256 _userColBefore = _collateral.balanceOf(_user);
    _executeWithdraw(_user, _supplyAmount, _borrowNf);

    assertEq(_collateral.balanceOf(_user), _userColBefore + _supplyAmount);
  }

  /*///////////////////////////////////////////////////////////////
                    C. NULLIFIER STATE MACHINE
  //////////////////////////////////////////////////////////////*/

  function test_Nullifier_CannotReuseBorrowNullifier() external {
    _supplyCollateral(_user, _supplyAmount);
    _executeBorrow(_user, _borrowAmount, _borrowNf);

    bytes32[] memory _pubInputs = new bytes32[](1);
    _pubInputs[0] = _borrowNf;

    vm.prank(_user);
    vm.expectRevert(IOGBankContract.OGBank_NullifierAlreadyUsed.selector);
    _ogBank.borrow(hex'cafe', _pubInputs, _borrowAmount);
  }

  function test_Nullifier_CannotDoubleRepay() external {
    _supplyCollateral(_user, _supplyAmount);
    _executeBorrow(_user, _borrowAmount, _borrowNf);
    _executeRepay(_user, _borrowAmount, _borrowNf);

    // Mint extra tokens for second repay attempt
    _borrowToken.mint(_user, _borrowAmount);

    vm.startPrank(_user);
    _borrowToken.approve(address(_ogBank), _borrowAmount);
    vm.expectRevert(IOGBankContract.OGBank_NullifierAlreadyUsed.selector);
    _ogBank.repay(_borrowAmount, _borrowNf);
    vm.stopPrank();
  }

  function test_Nullifier_CannotDoubleWithdraw() external {
    // Extra collateral for 2 withdrawals worth
    _collateral.mint(_user, _supplyAmount);
    _supplyCollateral(_user, _supplyAmount * 2);
    _executeBorrow(_user, _borrowAmount, _borrowNf);
    _executeRepay(_user, _borrowAmount, _borrowNf);

    bytes32[] memory _pubInputs = new bytes32[](1);
    _pubInputs[0] = _borrowNf;

    // First withdraw succeeds
    vm.prank(_user);
    _ogBank.withdrawProof(hex'cafe', _pubInputs, _supplyAmount);

    // Second withdraw reverts
    vm.prank(_user);
    vm.expectRevert(IOGBankContract.OGBank_AlreadyConsumed.selector);
    _ogBank.withdrawProof(hex'cafe', _pubInputs, _supplyAmount);
  }

  function test_Nullifier_CannotWithdrawWithoutRepay() external {
    _supplyCollateral(_user, _supplyAmount);
    _executeBorrow(_user, _borrowAmount, _borrowNf);
    // No repay!

    bytes32[] memory _pubInputs = new bytes32[](1);
    _pubInputs[0] = _borrowNf;

    vm.prank(_user);
    vm.expectRevert(IOGBankContract.OGBank_NotRepaid.selector);
    _ogBank.withdrawProof(hex'cafe', _pubInputs, _supplyAmount);
  }

  /*///////////////////////////////////////////////////////////////
                D. INVALID PROOF (MockVerifier toggle)
  //////////////////////////////////////////////////////////////*/

  function test_Borrow_RevertsWhenVerifierReturnsFalse() external {
    _supplyCollateral(_user, _supplyAmount);
    _mockVerifier.setShouldVerify(false);

    bytes32[] memory _pubInputs = new bytes32[](1);
    _pubInputs[0] = _borrowNf;

    vm.prank(_user);
    vm.expectRevert(IOGBankContract.OGBank_InvalidProof.selector);
    _ogBank.borrow(hex'cafe', _pubInputs, _borrowAmount);
  }

  function test_Withdraw_RevertsWhenVerifierReturnsFalse() external {
    _supplyCollateral(_user, _supplyAmount);
    _executeBorrow(_user, _borrowAmount, _borrowNf);
    _executeRepay(_user, _borrowAmount, _borrowNf);

    _mockVerifier.setShouldVerify(false);

    bytes32[] memory _pubInputs = new bytes32[](1);
    _pubInputs[0] = _borrowNf;

    vm.prank(_user);
    vm.expectRevert(IOGBankContract.OGBank_InvalidProof.selector);
    _ogBank.withdrawProof(hex'cafe', _pubInputs, _supplyAmount);
  }

  /*///////////////////////////////////////////////////////////////
                  E. MULTI-USER / MULTI-CYCLE
  //////////////////////////////////////////////////////////////*/

  function test_MultipleBorrowCycles_IndependentNullifiers() external {
    _collateral.mint(_user, _supplyAmount);
    _supplyCollateral(_user, _supplyAmount * 2);

    // Cycle 1
    bytes32 _nf1 = keccak256('cycle_1');
    _executeBorrow(_user, _borrowAmount, _nf1);
    _executeRepay(_user, _borrowAmount, _nf1);
    _executeWithdraw(_user, _supplyAmount, _nf1);

    // Cycle 2 with different nullifier
    bytes32 _nf2 = keccak256('cycle_2');
    _executeBorrow(_user, _borrowAmount, _nf2);
    _executeRepay(_user, _borrowAmount, _nf2);
    _executeWithdraw(_user, _supplyAmount, _nf2);

    // Both consumed
    assertTrue(_ogBank.consumedNullifiers(_nf1));
    assertTrue(_ogBank.consumedNullifiers(_nf2));

    // User got all collateral back
    assertEq(_collateral.balanceOf(_user), _supplyAmount * 2);
  }

  function test_MultiUser_IndependentCycles() external {
    // User 1 cycle
    bytes32 _nf1 = keccak256('user1_borrow');
    _supplyCollateral(_user, _supplyAmount);
    _executeBorrow(_user, _borrowAmount, _nf1);
    _executeRepay(_user, _borrowAmount, _nf1);
    _executeWithdraw(_user, _supplyAmount, _nf1);

    // User 2 cycle
    bytes32 _nf2 = keccak256('user2_borrow');
    _supplyCollateral(_user2, _supplyAmount);
    _executeBorrow(_user2, _borrowAmount, _nf2);
    _executeRepay(_user2, _borrowAmount, _nf2);
    _executeWithdraw(_user2, _supplyAmount, _nf2);

    // Both consumed independently
    assertTrue(_ogBank.consumedNullifiers(_nf1));
    assertTrue(_ogBank.consumedNullifiers(_nf2));

    // Both users got collateral back
    assertEq(_collateral.balanceOf(_user), _supplyAmount);
    assertEq(_collateral.balanceOf(_user2), _supplyAmount);
  }

  /*///////////////////////////////////////////////////////////////
                            HELPERS
  //////////////////////////////////////////////////////////////*/

  function _supplyCollateral(address _depositor, uint256 _amount) internal {
    vm.startPrank(_depositor);
    _collateral.approve(address(_ogBank), _amount);
    _ogBank.supplyCollateral(_amount);
    vm.stopPrank();
  }

  function _executeBorrow(address _borrower, uint256 _amount, bytes32 _borrowNullifier) internal {
    bytes32[] memory _pubInputs = new bytes32[](1);
    _pubInputs[0] = _borrowNullifier;

    vm.prank(_borrower);
    _ogBank.borrow(hex'cafe', _pubInputs, _amount);
  }

  function _executeRepay(address _repayer, uint256 _amount, bytes32 _borrowNullifier) internal {
    vm.startPrank(_repayer);
    _borrowToken.approve(address(_ogBank), _amount);
    _ogBank.repay(_amount, _borrowNullifier);
    vm.stopPrank();
  }

  function _executeWithdraw(address _withdrawer, uint256 _amount, bytes32 _borrowNullifier) internal {
    bytes32[] memory _pubInputs = new bytes32[](1);
    _pubInputs[0] = _borrowNullifier;

    vm.prank(_withdrawer);
    _ogBank.withdrawProof(hex'cafe', _pubInputs, _amount);
  }
}
