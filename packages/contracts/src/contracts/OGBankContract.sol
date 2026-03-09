// SPDX-License-Identifier: MIT
pragma solidity 0.8.34;

import {IAavePool} from 'interfaces/IAavePool.sol';
import {IOGBankContract} from 'interfaces/IOGBankContract.sol';
import {IERC20} from 'forge-std/interfaces/IERC20.sol';

/**
 * @title IVerifier
 * @notice Minimal interface for the Ultrahonk ZK verifier
 */
interface IVerifier {
  function verify(bytes calldata _proof, bytes32[] calldata _publicInputs) external returns (bool);
}

/**
 * @title OGBankContract
 * @notice Core orchestrator for OGBank: supply collateral → borrow → repay → withdraw
 * @dev Integrates with Aave V3 Pool for lending and the Ultrahonk Verifier for ZK proofs.
 *   Manages a nullifier chain (borrow → repay → consumed) to prevent replay attacks.
 */
contract OGBankContract is IOGBankContract {
  /*///////////////////////////////////////////////////////////////
                            CONSTANTS
  //////////////////////////////////////////////////////////////*/

  /// @notice Variable interest rate mode for Aave V3
  uint256 internal constant _VARIABLE_RATE = 2;

  /// @notice No referral code
  uint16 internal constant _NO_REFERRAL = 0;

  /*///////////////////////////////////////////////////////////////
                          IMMUTABLES
  //////////////////////////////////////////////////////////////*/

  /// @inheritdoc IOGBankContract
  address public immutable OWNER;

  /// @inheritdoc IOGBankContract
  address public immutable VERIFIER;

  /// @inheritdoc IOGBankContract
  address public immutable AAVE_POOL;

  /// @inheritdoc IOGBankContract
  address public immutable COLLATERAL_TOKEN;

  /// @inheritdoc IOGBankContract
  address public immutable BORROW_TOKEN;

  /*///////////////////////////////////////////////////////////////
                        NULLIFIER STORAGE
  //////////////////////////////////////////////////////////////*/

  /// @inheritdoc IOGBankContract
  mapping(bytes32 => bool) public borrowNullifiers;

  /// @inheritdoc IOGBankContract
  mapping(bytes32 => bool) public repayNullifiers;

  /// @inheritdoc IOGBankContract
  mapping(bytes32 => bool) public consumedNullifiers;

  /*///////////////////////////////////////////////////////////////
                          MODIFIERS
  //////////////////////////////////////////////////////////////*/

  modifier onlyOwner() {
    if (msg.sender != OWNER) revert OGBank_OnlyOwner();
    _;
  }

  /*///////////////////////////////////////////////////////////////
                          CONSTRUCTOR
  //////////////////////////////////////////////////////////////*/

  /**
   * @notice Initializes the OGBank contract
   * @param _owner Address of the protocol owner (treasury)
   * @param _verifier Address of the Ultrahonk ZK verifier contract
   * @param _aavePool Address of the Aave V3 Pool
   * @param _collateralToken Address of the collateral ERC20 token
   * @param _borrowToken Address of the ERC20 token to borrow
   */
  constructor(address _owner, address _verifier, address _aavePool, address _collateralToken, address _borrowToken) {
    OWNER = _owner;
    VERIFIER = _verifier;
    AAVE_POOL = _aavePool;
    COLLATERAL_TOKEN = _collateralToken;
    BORROW_TOKEN = _borrowToken;
  }

  /*///////////////////////////////////////////////////////////////
                              LOGIC
  //////////////////////////////////////////////////////////////*/

  /// @inheritdoc IOGBankContract
  function supplyCollateral(uint256 _amount) external onlyOwner {
    if (_amount == 0) revert OGBank_ZeroAmount();

    IERC20(COLLATERAL_TOKEN).transferFrom(msg.sender, address(this), _amount);
    IERC20(COLLATERAL_TOKEN).approve(AAVE_POOL, _amount);
    IAavePool(AAVE_POOL).supply(COLLATERAL_TOKEN, _amount, address(this), _NO_REFERRAL);

    emit CollateralSupplied(msg.sender, _amount);
  }

  /// @inheritdoc IOGBankContract
  function borrow(bytes calldata _proof, bytes32[] calldata _publicInputs, uint256 _amount) external {
    if (_amount == 0) revert OGBank_ZeroAmount();

    // Verify the ZK proof
    bool _valid = IVerifier(VERIFIER).verify(_proof, _publicInputs);
    if (!_valid) revert OGBank_InvalidProof();

    // Extract borrow nullifier from public inputs
    bytes32 _borrowNullifier = _publicInputs[0];
    if (borrowNullifiers[_borrowNullifier]) revert OGBank_NullifierAlreadyUsed();

    // Register the borrow nullifier
    borrowNullifiers[_borrowNullifier] = true;

    // Borrow from Aave V3
    IAavePool(AAVE_POOL).borrow(BORROW_TOKEN, _amount, _VARIABLE_RATE, _NO_REFERRAL, address(this));

    // Transfer borrowed tokens to the user
    IERC20(BORROW_TOKEN).transfer(msg.sender, _amount);

    emit BorrowExecuted(msg.sender, _amount, _borrowNullifier);
  }

  /// @inheritdoc IOGBankContract
  function repay(uint256 _amount, bytes32 _borrowNullifier) external {
    if (_amount == 0) revert OGBank_ZeroAmount();
    if (!borrowNullifiers[_borrowNullifier]) revert OGBank_BorrowNullifierNotFound();

    // Derive the repay nullifier from the borrow nullifier
    bytes32 _repayNullifier = keccak256(abi.encodePacked(_borrowNullifier, msg.sender));
    if (repayNullifiers[_repayNullifier]) revert OGBank_NullifierAlreadyUsed();

    // Transfer tokens from user to this contract
    IERC20(BORROW_TOKEN).transferFrom(msg.sender, address(this), _amount);

    // Approve and repay to Aave V3
    IERC20(BORROW_TOKEN).approve(AAVE_POOL, _amount);
    IAavePool(AAVE_POOL).repay(BORROW_TOKEN, _amount, _VARIABLE_RATE, address(this));

    // Register the repay nullifier
    repayNullifiers[_repayNullifier] = true;

    emit RepayExecuted(msg.sender, _amount, _borrowNullifier);
  }

  /// @inheritdoc IOGBankContract
  function withdrawProof(bytes calldata _proof, bytes32[] calldata _publicInputs, uint256 _amount) external {
    if (_amount == 0) revert OGBank_ZeroAmount();

    // Verify the ZK proof
    bool _valid = IVerifier(VERIFIER).verify(_proof, _publicInputs);
    if (!_valid) revert OGBank_InvalidProof();

    // Extract nullifiers from public inputs
    bytes32 _borrowNullifier = _publicInputs[0];

    // Verify nullifier chain: borrow exists, repay exists, not yet consumed
    if (!borrowNullifiers[_borrowNullifier]) revert OGBank_BorrowNullifierNotFound();

    bytes32 _repayNullifier = keccak256(abi.encodePacked(_borrowNullifier, msg.sender));
    if (!repayNullifiers[_repayNullifier]) revert OGBank_NotRepaid();

    if (consumedNullifiers[_borrowNullifier]) revert OGBank_AlreadyConsumed();

    // Mark as consumed
    consumedNullifiers[_borrowNullifier] = true;

    // Emit event for the relayer to return ZEC on Zcash
    emit FinishPayment(address(this), _amount, msg.sender, msg.sender);
  }
}
