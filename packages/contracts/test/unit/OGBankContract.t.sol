// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.34;

import {IOGBankContract} from 'interfaces/IOGBankContract.sol';
import {MockAavePool} from 'contracts/mocks/MockAavePool.sol';
import {OGBankContract, IVerifier} from 'contracts/OGBankContract.sol';
import {Test} from 'forge-std/Test.sol';

import {MockERC20} from './helpers/MockERC20.sol';

contract UnitOGBankContract is Test {
  OGBankContract internal _ogBank;
  MockAavePool internal _aavePool;
  MockERC20 internal _collateral;
  MockERC20 internal _borrowToken;
  address internal _verifier;

  address internal _user = makeAddr('user');
  uint256 internal _supplyAmount = 1000e18;
  uint256 internal _borrowAmount = 500e18;
  bytes32 internal _testBorrowNullifier = keccak256('borrow_nullifier_1');

  event CollateralSupplied(address indexed _user, uint256 _amount);
  event BorrowExecuted(address indexed _user, uint256 _amount, bytes32 indexed _borrowNullifier);
  event RepayExecuted(address indexed _user, uint256 _amount, bytes32 indexed _borrowNullifier);
  event FinishPayment(address indexed _ogbank, uint256 _amount, address _recipient, address _originAddress);

  function setUp() external {
    _verifier = makeAddr('verifier');
    _aavePool = new MockAavePool();
    _collateral = new MockERC20('Collateral', 'COL');
    _borrowToken = new MockERC20('BorrowToken', 'BOR');

    _ogBank = new OGBankContract(
      _verifier,
      address(_aavePool),
      address(_collateral),
      address(_borrowToken)
    );

    // Fund the user with collateral
    _collateral.mint(_user, _supplyAmount);

    // Fund the Aave pool with borrow tokens so it can lend
    _borrowToken.mint(address(_aavePool), _borrowAmount * 10);

    // Default: mock the verifier to return true
    vm.mockCall(_verifier, abi.encodeWithSelector(IVerifier.verify.selector), abi.encode(true));
  }

  /*///////////////////////////////////////////////////////////////
                          CONSTRUCTOR
  //////////////////////////////////////////////////////////////*/

  function test_Constructor_SetsImmutables() external view {
    assertEq(_ogBank.VERIFIER(), _verifier);
    assertEq(_ogBank.AAVE_POOL(), address(_aavePool));
    assertEq(_ogBank.COLLATERAL_TOKEN(), address(_collateral));
    assertEq(_ogBank.BORROW_TOKEN(), address(_borrowToken));
  }

  /*///////////////////////////////////////////////////////////////
                        SUPPLY COLLATERAL
  //////////////////////////////////////////////////////////////*/

  function test_SupplyCollateral_TransfersToAave() external {
    vm.startPrank(_user);
    _collateral.approve(address(_ogBank), _supplyAmount);
    _ogBank.supplyCollateral(_supplyAmount);
    vm.stopPrank();

    // Collateral should be in the Aave pool
    assertEq(_collateral.balanceOf(address(_aavePool)), _supplyAmount);
    assertEq(_collateral.balanceOf(_user), 0);
    // Aave pool should track it for OGBank
    assertEq(_aavePool.supplied(address(_ogBank), address(_collateral)), _supplyAmount);
  }

  function test_SupplyCollateral_EmitsEvent() external {
    vm.startPrank(_user);
    _collateral.approve(address(_ogBank), _supplyAmount);

    vm.expectEmit(true, true, true, true, address(_ogBank));
    emit CollateralSupplied(_user, _supplyAmount);

    _ogBank.supplyCollateral(_supplyAmount);
    vm.stopPrank();
  }

  function test_SupplyCollateral_RevertsOnZeroAmount() external {
    vm.prank(_user);
    vm.expectRevert(IOGBankContract.OGBank_ZeroAmount.selector);
    _ogBank.supplyCollateral(0);
  }

  /*///////////////////////////////////////////////////////////////
                              BORROW
  //////////////////////////////////////////////////////////////*/

  function test_Borrow_WithValidProof() external {
    // Setup: supply collateral first
    _supplyCollateral(_user, _supplyAmount);

    bytes32[] memory _publicInputs = new bytes32[](1);
    _publicInputs[0] = _testBorrowNullifier;

    vm.prank(_user);
    _ogBank.borrow(hex'deadbeef', _publicInputs, _borrowAmount);

    // User should have received borrow tokens
    assertEq(_borrowToken.balanceOf(_user), _borrowAmount);
    // Nullifier should be registered
    assertTrue(_ogBank.borrowNullifiers(_testBorrowNullifier));
    // Aave should track the borrow for OGBank
    assertEq(_aavePool.borrowed(address(_ogBank), address(_borrowToken)), _borrowAmount);
  }

  function test_Borrow_EmitsEvent() external {
    _supplyCollateral(_user, _supplyAmount);

    bytes32[] memory _publicInputs = new bytes32[](1);
    _publicInputs[0] = _testBorrowNullifier;

    vm.expectEmit(true, true, true, true, address(_ogBank));
    emit BorrowExecuted(_user, _borrowAmount, _testBorrowNullifier);

    vm.prank(_user);
    _ogBank.borrow(hex'deadbeef', _publicInputs, _borrowAmount);
  }

  function test_Borrow_RevertsOnInvalidProof() external {
    _supplyCollateral(_user, _supplyAmount);

    // Mock verifier to return false
    vm.mockCall(_verifier, abi.encodeWithSelector(IVerifier.verify.selector), abi.encode(false));

    bytes32[] memory _publicInputs = new bytes32[](1);
    _publicInputs[0] = _testBorrowNullifier;

    vm.prank(_user);
    vm.expectRevert(IOGBankContract.OGBank_InvalidProof.selector);
    _ogBank.borrow(hex'deadbeef', _publicInputs, _borrowAmount);
  }

  function test_Borrow_RevertsOnDuplicateNullifier() external {
    _supplyCollateral(_user, _supplyAmount);

    bytes32[] memory _publicInputs = new bytes32[](1);
    _publicInputs[0] = _testBorrowNullifier;

    // First borrow succeeds
    vm.prank(_user);
    _ogBank.borrow(hex'deadbeef', _publicInputs, _borrowAmount);

    // Second borrow with same nullifier reverts
    vm.prank(_user);
    vm.expectRevert(IOGBankContract.OGBank_NullifierAlreadyUsed.selector);
    _ogBank.borrow(hex'deadbeef', _publicInputs, _borrowAmount);
  }

  function test_Borrow_RevertsOnZeroAmount() external {
    bytes32[] memory _publicInputs = new bytes32[](1);
    _publicInputs[0] = _testBorrowNullifier;

    vm.prank(_user);
    vm.expectRevert(IOGBankContract.OGBank_ZeroAmount.selector);
    _ogBank.borrow(hex'deadbeef', _publicInputs, 0);
  }

  /*///////////////////////////////////////////////////////////////
                              REPAY
  //////////////////////////////////////////////////////////////*/

  function test_Repay_ReducesDebt() external {
    _supplyCollateral(_user, _supplyAmount);
    _executeBorrow(_user, _borrowAmount, _testBorrowNullifier);

    // User approves OGBank to pull borrow tokens for repay
    vm.startPrank(_user);
    _borrowToken.approve(address(_ogBank), _borrowAmount);
    _ogBank.repay(_borrowAmount, _testBorrowNullifier);
    vm.stopPrank();

    // Repay nullifier should be registered
    bytes32 _repayNullifier = keccak256(abi.encodePacked(_testBorrowNullifier, _user));
    assertTrue(_ogBank.repayNullifiers(_repayNullifier));
  }

  function test_Repay_EmitsEvent() external {
    _supplyCollateral(_user, _supplyAmount);
    _executeBorrow(_user, _borrowAmount, _testBorrowNullifier);

    vm.startPrank(_user);
    _borrowToken.approve(address(_ogBank), _borrowAmount);

    vm.expectEmit(true, true, true, true, address(_ogBank));
    emit RepayExecuted(_user, _borrowAmount, _testBorrowNullifier);

    _ogBank.repay(_borrowAmount, _testBorrowNullifier);
    vm.stopPrank();
  }

  function test_Repay_RevertsOnInvalidBorrowNullifier() external {
    bytes32 _fakeBorrowNullifier = keccak256('fake');

    vm.prank(_user);
    vm.expectRevert(IOGBankContract.OGBank_BorrowNullifierNotFound.selector);
    _ogBank.repay(_borrowAmount, _fakeBorrowNullifier);
  }

  function test_Repay_RevertsOnDuplicateRepay() external {
    _supplyCollateral(_user, _supplyAmount);
    _executeBorrow(_user, _borrowAmount, _testBorrowNullifier);

    // Mint extra tokens for double repay attempt
    _borrowToken.mint(_user, _borrowAmount);

    vm.startPrank(_user);
    _borrowToken.approve(address(_ogBank), _borrowAmount * 2);

    // First repay succeeds
    _ogBank.repay(_borrowAmount, _testBorrowNullifier);

    // Second repay with same nullifier reverts
    vm.expectRevert(IOGBankContract.OGBank_NullifierAlreadyUsed.selector);
    _ogBank.repay(_borrowAmount, _testBorrowNullifier);
    vm.stopPrank();
  }

  function test_Repay_RevertsOnZeroAmount() external {
    vm.prank(_user);
    vm.expectRevert(IOGBankContract.OGBank_ZeroAmount.selector);
    _ogBank.repay(0, _testBorrowNullifier);
  }

  /*///////////////////////////////////////////////////////////////
                          WITHDRAW PROOF
  //////////////////////////////////////////////////////////////*/

  function test_WithdrawProof_FullCycle() external {
    // 1. Supply collateral
    _supplyCollateral(_user, _supplyAmount);

    // 2. Borrow
    _executeBorrow(_user, _borrowAmount, _testBorrowNullifier);

    // 3. Repay
    _executeRepay(_user, _borrowAmount, _testBorrowNullifier);

    // 4. Withdraw with proof
    bytes32[] memory _publicInputs = new bytes32[](1);
    _publicInputs[0] = _testBorrowNullifier;

    vm.prank(_user);
    _ogBank.withdrawProof(hex'deadbeef', _publicInputs, _supplyAmount);

    // Nullifier should be consumed
    assertTrue(_ogBank.consumedNullifiers(_testBorrowNullifier));
    // User should have received collateral back
    assertEq(_collateral.balanceOf(_user), _supplyAmount);
  }

  function test_WithdrawProof_EmitsFinishPayment() external {
    _supplyCollateral(_user, _supplyAmount);
    _executeBorrow(_user, _borrowAmount, _testBorrowNullifier);
    _executeRepay(_user, _borrowAmount, _testBorrowNullifier);

    bytes32[] memory _publicInputs = new bytes32[](1);
    _publicInputs[0] = _testBorrowNullifier;

    vm.expectEmit(true, true, true, true, address(_ogBank));
    emit FinishPayment(address(_ogBank), _supplyAmount, _user, _user);

    vm.prank(_user);
    _ogBank.withdrawProof(hex'deadbeef', _publicInputs, _supplyAmount);
  }

  function test_WithdrawProof_RevertsOnInvalidProof() external {
    _supplyCollateral(_user, _supplyAmount);
    _executeBorrow(_user, _borrowAmount, _testBorrowNullifier);
    _executeRepay(_user, _borrowAmount, _testBorrowNullifier);

    vm.mockCall(_verifier, abi.encodeWithSelector(IVerifier.verify.selector), abi.encode(false));

    bytes32[] memory _publicInputs = new bytes32[](1);
    _publicInputs[0] = _testBorrowNullifier;

    vm.prank(_user);
    vm.expectRevert(IOGBankContract.OGBank_InvalidProof.selector);
    _ogBank.withdrawProof(hex'deadbeef', _publicInputs, _supplyAmount);
  }

  function test_WithdrawProof_RevertsOnMissingBorrowNullifier() external {
    bytes32 _fakeBorrowNullifier = keccak256('no_borrow');
    bytes32[] memory _publicInputs = new bytes32[](1);
    _publicInputs[0] = _fakeBorrowNullifier;

    vm.prank(_user);
    vm.expectRevert(IOGBankContract.OGBank_BorrowNullifierNotFound.selector);
    _ogBank.withdrawProof(hex'deadbeef', _publicInputs, _supplyAmount);
  }

  function test_WithdrawProof_RevertsOnMissingRepay() external {
    _supplyCollateral(_user, _supplyAmount);
    _executeBorrow(_user, _borrowAmount, _testBorrowNullifier);
    // No repay!

    bytes32[] memory _publicInputs = new bytes32[](1);
    _publicInputs[0] = _testBorrowNullifier;

    vm.prank(_user);
    vm.expectRevert(IOGBankContract.OGBank_NotRepaid.selector);
    _ogBank.withdrawProof(hex'deadbeef', _publicInputs, _supplyAmount);
  }

  function test_WithdrawProof_RevertsOnAlreadyConsumed() external {
    _collateral.mint(_user, _supplyAmount);
    _supplyCollateral(_user, _supplyAmount * 2);
    _executeBorrow(_user, _borrowAmount, _testBorrowNullifier);
    _executeRepay(_user, _borrowAmount, _testBorrowNullifier);

    bytes32[] memory _publicInputs = new bytes32[](1);
    _publicInputs[0] = _testBorrowNullifier;

    // First withdraw succeeds
    vm.prank(_user);
    _ogBank.withdrawProof(hex'deadbeef', _publicInputs, _supplyAmount);

    // Second withdraw with same nullifier reverts
    vm.prank(_user);
    vm.expectRevert(IOGBankContract.OGBank_AlreadyConsumed.selector);
    _ogBank.withdrawProof(hex'deadbeef', _publicInputs, _supplyAmount);
  }

  function test_WithdrawProof_RevertsOnZeroAmount() external {
    bytes32[] memory _publicInputs = new bytes32[](1);
    _publicInputs[0] = _testBorrowNullifier;

    vm.prank(_user);
    vm.expectRevert(IOGBankContract.OGBank_ZeroAmount.selector);
    _ogBank.withdrawProof(hex'deadbeef', _publicInputs, 0);
  }

  /*///////////////////////////////////////////////////////////////
                      FULL NULLIFIER CHAIN
  //////////////////////////////////////////////////////////////*/

  function test_FullNullifierChain_BorrowRepayWithdraw() external {
    // Supply collateral
    _supplyCollateral(_user, _supplyAmount);

    // Borrow — creates borrowNullifier
    bytes32 _borrowNf = keccak256('cycle_1');
    _executeBorrow(_user, _borrowAmount, _borrowNf);

    assertTrue(_ogBank.borrowNullifiers(_borrowNf));
    assertFalse(_ogBank.consumedNullifiers(_borrowNf));

    // Repay — creates repayNullifier linked to borrowNullifier
    _executeRepay(_user, _borrowAmount, _borrowNf);

    bytes32 _repayNf = keccak256(abi.encodePacked(_borrowNf, _user));
    assertTrue(_ogBank.repayNullifiers(_repayNf));

    // Withdraw — consumes borrowNullifier
    bytes32[] memory _publicInputs = new bytes32[](1);
    _publicInputs[0] = _borrowNf;

    vm.prank(_user);
    _ogBank.withdrawProof(hex'deadbeef', _publicInputs, _supplyAmount);

    assertTrue(_ogBank.consumedNullifiers(_borrowNf));
  }

  function test_MultipleBorrowCycles() external {
    // Fund user with extra collateral and borrow tokens
    _collateral.mint(_user, _supplyAmount);
    _supplyCollateral(_user, _supplyAmount * 2);

    // Cycle 1
    bytes32 _borrowNf1 = keccak256('cycle_a');
    _executeBorrow(_user, _borrowAmount, _borrowNf1);
    _executeRepay(_user, _borrowAmount, _borrowNf1);

    bytes32[] memory _publicInputs1 = new bytes32[](1);
    _publicInputs1[0] = _borrowNf1;
    vm.prank(_user);
    _ogBank.withdrawProof(hex'deadbeef', _publicInputs1, _supplyAmount);

    // Cycle 2 with different nullifier
    bytes32 _borrowNf2 = keccak256('cycle_b');

    // Fund pool with more borrow tokens for second cycle
    _borrowToken.mint(address(_aavePool), _borrowAmount * 10);

    _executeBorrow(_user, _borrowAmount, _borrowNf2);
    _executeRepay(_user, _borrowAmount, _borrowNf2);

    bytes32[] memory _publicInputs2 = new bytes32[](1);
    _publicInputs2[0] = _borrowNf2;
    vm.prank(_user);
    _ogBank.withdrawProof(hex'deadbeef', _publicInputs2, _supplyAmount);

    // Both nullifiers should be consumed
    assertTrue(_ogBank.consumedNullifiers(_borrowNf1));
    assertTrue(_ogBank.consumedNullifiers(_borrowNf2));
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
    bytes32[] memory _publicInputs = new bytes32[](1);
    _publicInputs[0] = _borrowNullifier;

    vm.prank(_borrower);
    _ogBank.borrow(hex'deadbeef', _publicInputs, _amount);
  }

  function _executeRepay(address _repayer, uint256 _amount, bytes32 _borrowNullifier) internal {
    vm.startPrank(_repayer);
    _borrowToken.approve(address(_ogBank), _amount);
    _ogBank.repay(_amount, _borrowNullifier);
    vm.stopPrank();
  }
}
