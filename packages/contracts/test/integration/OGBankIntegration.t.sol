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
      address(this),
      address(_mockVerifier),
      address(_aavePool),
      address(_collateral),
      address(_borrowToken)
    );

    // Fund owner (this contract) with collateral for protocol treasury
    _collateral.mint(address(this), _supplyAmount * 5);
    // Fund Aave pool with borrow tokens
    _borrowToken.mint(address(_aavePool), _borrowAmount * 20);
  }

  /*///////////////////////////////////////////////////////////////
                    A. FULL LIFECYCLE WITH BALANCES
  //////////////////////////////////////////////////////////////*/

  function test_FullCycle_BalancesAtEveryStep() external {
    // -- Pre-state: user has no tokens --
    assertEq(_borrowToken.balanceOf(_user), 0);

    // 1. Supply collateral (owner)
    _supplyCollateral(_supplyAmount);

    assertEq(_collateral.balanceOf(address(_aavePool)), _supplyAmount, 'aave collateral after supply');
    assertEq(_aavePool.supplied(address(_ogBank), address(_collateral)), _supplyAmount, 'aave tracked supply');

    // 2. Borrow (user)
    _executeBorrow(_user, _borrowAmount, _borrowNf);

    assertEq(_borrowToken.balanceOf(_user), _borrowAmount, 'user borrow tokens after borrow');
    assertEq(_aavePool.borrowed(address(_ogBank), address(_borrowToken)), _borrowAmount, 'aave tracked borrow');

    // 3. Repay (user)
    _executeRepay(_user, _borrowAmount, _borrowNf);

    assertEq(_borrowToken.balanceOf(_user), 0, 'user borrow tokens after repay');
    assertEq(_aavePool.borrowed(address(_ogBank), address(_borrowToken)), 0, 'aave debt after repay');

    // 4. Withdraw (user) — no collateral transfer, just event for relayer
    _executeWithdraw(_user, _supplyAmount, _borrowNf);

    // Collateral stays in Aave (protocol treasury)
    assertEq(_collateral.balanceOf(_user), 0, 'user has no collateral');
  }

  function test_FullCycle_NullifierChainState() external {
    _supplyCollateral(_supplyAmount);

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
    // Supply event (owner)
    _collateral.approve(address(_ogBank), _supplyAmount);
    vm.expectEmit(true, true, true, true, address(_ogBank));
    emit CollateralSupplied(address(this), _supplyAmount);
    _ogBank.supplyCollateral(_supplyAmount);

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

    // Withdraw event — signals relayer
    vm.expectEmit(true, true, true, true, address(_ogBank));
    emit FinishPayment(address(_ogBank), _supplyAmount, _user, _user);
    vm.prank(_user);
    _ogBank.withdrawProof(hex'cafe', _pubInputs, _supplyAmount);
  }

  /*///////////////////////////////////////////////////////////////
                      B. BALANCE ACCOUNTING
  //////////////////////////////////////////////////////////////*/

  function test_Balances_SupplyFromOwnerToAave() external {
    uint256 _ownerBefore = _collateral.balanceOf(address(this));
    uint256 _aaveBefore = _collateral.balanceOf(address(_aavePool));

    _supplyCollateral(_supplyAmount);

    assertEq(_collateral.balanceOf(address(this)), _ownerBefore - _supplyAmount);
    assertEq(_collateral.balanceOf(address(_aavePool)), _aaveBefore + _supplyAmount);
  }

  function test_Balances_BorrowGivesUserTokens() external {
    _supplyCollateral(_supplyAmount);

    uint256 _userBorrowBefore = _borrowToken.balanceOf(_user);
    _executeBorrow(_user, _borrowAmount, _borrowNf);

    assertEq(_borrowToken.balanceOf(_user), _userBorrowBefore + _borrowAmount);
    assertEq(_aavePool.borrowed(address(_ogBank), address(_borrowToken)), _borrowAmount);
  }

  function test_Balances_RepayPullsUserTokens() external {
    _supplyCollateral(_supplyAmount);
    _executeBorrow(_user, _borrowAmount, _borrowNf);

    uint256 _userBorrowBefore = _borrowToken.balanceOf(_user);
    _executeRepay(_user, _borrowAmount, _borrowNf);

    assertEq(_borrowToken.balanceOf(_user), _userBorrowBefore - _borrowAmount);
    assertEq(_aavePool.borrowed(address(_ogBank), address(_borrowToken)), 0);
  }

  function test_Balances_WithdrawDoesNotTransferCollateral() external {
    _supplyCollateral(_supplyAmount);
    _executeBorrow(_user, _borrowAmount, _borrowNf);
    _executeRepay(_user, _borrowAmount, _borrowNf);

    uint256 _userColBefore = _collateral.balanceOf(_user);
    _executeWithdraw(_user, _supplyAmount, _borrowNf);

    // User never receives collateral — it stays in Aave
    assertEq(_collateral.balanceOf(_user), _userColBefore);
  }

  /*///////////////////////////////////////////////////////////////
                    C. NULLIFIER STATE MACHINE
  //////////////////////////////////////////////////////////////*/

  function test_Nullifier_CannotReuseBorrowNullifier() external {
    _supplyCollateral(_supplyAmount);
    _executeBorrow(_user, _borrowAmount, _borrowNf);

    bytes32[] memory _pubInputs = new bytes32[](1);
    _pubInputs[0] = _borrowNf;

    vm.prank(_user);
    vm.expectRevert(IOGBankContract.OGBank_NullifierAlreadyUsed.selector);
    _ogBank.borrow(hex'cafe', _pubInputs, _borrowAmount);
  }

  function test_Nullifier_CannotDoubleRepay() external {
    _supplyCollateral(_supplyAmount);
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
    _supplyCollateral(_supplyAmount);
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
    _supplyCollateral(_supplyAmount);
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
    _supplyCollateral(_supplyAmount);
    _mockVerifier.setShouldVerify(false);

    bytes32[] memory _pubInputs = new bytes32[](1);
    _pubInputs[0] = _borrowNf;

    vm.prank(_user);
    vm.expectRevert(IOGBankContract.OGBank_InvalidProof.selector);
    _ogBank.borrow(hex'cafe', _pubInputs, _borrowAmount);
  }

  function test_Withdraw_RevertsWhenVerifierReturnsFalse() external {
    _supplyCollateral(_supplyAmount);
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
    _supplyCollateral(_supplyAmount * 2);

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
  }

  function test_MultiUser_IndependentCycles() external {
    // Protocol supplies collateral once
    _supplyCollateral(_supplyAmount * 2);

    // User 1 cycle
    bytes32 _nf1 = keccak256('user1_borrow');
    _executeBorrow(_user, _borrowAmount, _nf1);
    _executeRepay(_user, _borrowAmount, _nf1);
    _executeWithdraw(_user, _supplyAmount, _nf1);

    // User 2 cycle
    bytes32 _nf2 = keccak256('user2_borrow');
    _executeBorrow(_user2, _borrowAmount, _nf2);
    _executeRepay(_user2, _borrowAmount, _nf2);
    _executeWithdraw(_user2, _supplyAmount, _nf2);

    // Both consumed independently
    assertTrue(_ogBank.consumedNullifiers(_nf1));
    assertTrue(_ogBank.consumedNullifiers(_nf2));
  }

  /*///////////////////////////////////////////////////////////////
                            HELPERS
  //////////////////////////////////////////////////////////////*/

  function _supplyCollateral(uint256 _amount) internal {
    _collateral.approve(address(_ogBank), _amount);
    _ogBank.supplyCollateral(_amount);
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
