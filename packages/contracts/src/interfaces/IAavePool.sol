// SPDX-License-Identifier: MIT
pragma solidity 0.8.34;

/**
 * @title IAavePool
 * @notice Minimal interface for Aave V3 Pool — only the functions OGBank uses
 */
interface IAavePool {
  /**
   * @notice Supply assets to the Aave pool
   * @param _asset The address of the underlying asset to supply
   * @param _amount The amount to supply
   * @param _onBehalfOf The address that will receive the aTokens
   * @param _referralCode Referral code (0 for no referral)
   */
  function supply(address _asset, uint256 _amount, address _onBehalfOf, uint16 _referralCode) external;

  /**
   * @notice Borrow assets from the Aave pool
   * @param _asset The address of the underlying asset to borrow
   * @param _amount The amount to borrow
   * @param _interestRateMode 1 for stable, 2 for variable
   * @param _referralCode Referral code (0 for no referral)
   * @param _onBehalfOf The address that will receive the debt tokens
   */
  function borrow(
    address _asset,
    uint256 _amount,
    uint256 _interestRateMode,
    uint16 _referralCode,
    address _onBehalfOf
  ) external;

  /**
   * @notice Repay borrowed assets to the Aave pool
   * @param _asset The address of the underlying asset to repay
   * @param _amount The amount to repay (use type(uint256).max to repay the full debt)
   * @param _interestRateMode 1 for stable, 2 for variable
   * @param _onBehalfOf The address of the user who will get their debt reduced
   * @return The final amount repaid
   */
  function repay(address _asset, uint256 _amount, uint256 _interestRateMode, address _onBehalfOf)
    external
    returns (uint256);

  /**
   * @notice Withdraw assets from the Aave pool
   * @param _asset The address of the underlying asset to withdraw
   * @param _amount The amount to withdraw (use type(uint256).max to withdraw the full balance)
   * @param _to The address that will receive the underlying asset
   * @return The final amount withdrawn
   */
  function withdraw(address _asset, uint256 _amount, address _to) external returns (uint256);
}
