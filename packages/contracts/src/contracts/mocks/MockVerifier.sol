// SPDX-License-Identifier: MIT
pragma solidity 0.8.34;

import {IVerifier} from 'contracts/OGBankContract.sol';

/**
 * @title MockVerifier
 * @notice Mock ZK verifier for integration testing — toggleable pass/fail
 */
contract MockVerifier is IVerifier {
  bool public shouldVerify = true;

  function verify(bytes calldata, bytes32[] calldata) external view returns (bool) {
    return shouldVerify;
  }

  function setShouldVerify(bool _value) external {
    shouldVerify = _value;
  }
}
