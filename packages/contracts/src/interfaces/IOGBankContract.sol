// SPDX-License-Identifier: MIT
pragma solidity 0.8.34;

/**
 * @title IOGBankContract
 * @notice Interface for the OGBank core orchestrator contract
 * @dev Manages the full borrow → repay → withdraw cycle with ZK proof verification
 * and Aave V3 integration for lending operations.
 */
interface IOGBankContract {
  /*///////////////////////////////////////////////////////////////
                              EVENTS
  //////////////////////////////////////////////////////////////*/

  /**
   * @notice Emitted when collateral is supplied to Aave
   * @param _user The address that supplied collateral
   * @param _amount The amount of collateral supplied
   */
  event CollateralSupplied(address indexed _user, uint256 _amount);

  /**
   * @notice Emitted when a borrow is executed
   * @param _user The address that borrowed
   * @param _amount The amount borrowed
   * @param _borrowNullifier The unique nullifier for this borrow cycle
   */
  event BorrowExecuted(address indexed _user, uint256 _amount, bytes32 indexed _borrowNullifier);

  /**
   * @notice Emitted when a repayment is executed
   * @param _user The address that repaid
   * @param _amount The amount repaid
   * @param _borrowNullifier The borrow nullifier being repaid
   */
  event RepayExecuted(address indexed _user, uint256 _amount, bytes32 indexed _borrowNullifier);

  /**
   * @notice Emitted on successful withdrawal — triggers relayer to send ZEC
   * @param _ogbank The OGBank contract address
   * @param _amount The amount withdrawn
   * @param _recipient The recipient address
   * @param _originAddress The original ZCash address for ZEC return
   */
  event FinishPayment(address indexed _ogbank, uint256 _amount, address _recipient, address _originAddress);

  /*///////////////////////////////////////////////////////////////
                              ERRORS
  //////////////////////////////////////////////////////////////*/

  /// @notice Thrown when ZK proof verification fails
  error OGBank_InvalidProof();

  /// @notice Thrown when a borrow nullifier has already been used
  error OGBank_NullifierAlreadyUsed();

  /// @notice Thrown when a referenced borrow nullifier does not exist
  error OGBank_BorrowNullifierNotFound();

  /// @notice Thrown when attempting to withdraw without a valid repayment
  error OGBank_NotRepaid();

  /// @notice Thrown when the borrow nullifier has already been consumed (withdrawn)
  error OGBank_AlreadyConsumed();

  /// @notice Thrown when amount is zero
  error OGBank_ZeroAmount();

  /*///////////////////////////////////////////////////////////////
                            VARIABLES
  //////////////////////////////////////////////////////////////*/

  /**
   * @notice Returns the address of the ZK proof verifier
   * @return _verifier The verifier contract address
   */
  function VERIFIER() external view returns (address _verifier);

  /**
   * @notice Returns the address of the Aave V3 pool
   * @return _pool The Aave pool address
   */
  function AAVE_POOL() external view returns (address _pool);

  /**
   * @notice Returns the collateral token address
   * @return _token The collateral token
   */
  function COLLATERAL_TOKEN() external view returns (address _token);

  /**
   * @notice Returns the borrow token address
   * @return _token The borrow token
   */
  function BORROW_TOKEN() external view returns (address _token);

  /**
   * @notice Checks if a borrow nullifier has been registered
   * @param _nullifier The nullifier to check
   * @return _exists Whether the nullifier exists
   */
  function borrowNullifiers(bytes32 _nullifier) external view returns (bool _exists);

  /**
   * @notice Checks if a repay nullifier has been registered
   * @param _nullifier The nullifier to check
   * @return _exists Whether the nullifier exists
   */
  function repayNullifiers(bytes32 _nullifier) external view returns (bool _exists);

  /**
   * @notice Checks if a borrow nullifier has been consumed (withdrawn)
   * @param _nullifier The nullifier to check
   * @return _consumed Whether the nullifier has been consumed
   */
  function consumedNullifiers(bytes32 _nullifier) external view returns (bool _consumed);

  /*///////////////////////////////////////////////////////////////
                              LOGIC
  //////////////////////////////////////////////////////////////*/

  /**
   * @notice Supply collateral to the Aave V3 pool
   * @dev Transfers collateral from the caller, approves the Aave pool, and supplies it
   * @param _amount Amount of collateral to supply
   */
  function supplyCollateral(uint256 _amount) external;

  /**
   * @notice Borrow against verified collateral with a ZK proof
   * @dev Verifies the proof, creates a borrow nullifier, and borrows from Aave V3.
   *   The borrow nullifier is extracted from publicInputs[0].
   * @param _proof The Ultrahonk ZK proof bytes
   * @param _publicInputs The public inputs for proof verification
   * @param _amount Amount to borrow
   */
  function borrow(bytes calldata _proof, bytes32[] calldata _publicInputs, uint256 _amount) external;

  /**
   * @notice Repay borrowed tokens to the Aave V3 pool
   * @dev Links the repayment to a specific borrow via its nullifier.
   *   Creates a repay nullifier derived from the borrow nullifier.
   * @param _amount Amount to repay
   * @param _borrowNullifier The borrow nullifier to repay against
   */
  function repay(uint256 _amount, bytes32 _borrowNullifier) external;

  /**
   * @notice Withdraw collateral with proof of repayment
   * @dev Verifies the proof, checks the nullifier chain (borrow → repay → not consumed),
   *   withdraws from Aave V3, and emits FinishPayment for the relayer.
   *   The borrow nullifier is extracted from publicInputs[0].
   * @param _proof The Ultrahonk ZK proof bytes
   * @param _publicInputs The public inputs for proof verification
   * @param _amount Amount to withdraw
   */
  function withdrawProof(bytes calldata _proof, bytes32[] calldata _publicInputs, uint256 _amount) external;
}
