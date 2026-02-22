// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

/// @title ZLendMVP
/// @notice Minimal lending contract that trusts a relayer to attest ZEC collateral.
/// @dev The relayer calls `borrow()` directly — `msg.sender == trustedRelayer` is the auth check.
///      In production, this will be replaced by ZK proof verification (Ultrahonk).
contract ZLendMVP is Ownable {
    IERC20 public immutable borrowToken; // MockUSDC
    address public trustedRelayer;
    uint256 public constant COLLATERAL_RATIO = 150; // 150%

    struct Loan {
        bytes32 positionId;
        uint256 collateralZat; // ZEC collateral in zatoshis
        uint256 borrowedAmount; // ERC-20 tokens borrowed
        address borrower;
        bool active;
    }

    mapping(bytes32 => Loan) public loans;

    event Borrowed(
        bytes32 indexed positionId,
        address indexed borrower,
        uint256 collateralZat,
        uint256 amount
    );
    event RelayerUpdated(address indexed oldRelayer, address indexed newRelayer);

    constructor(
        address _borrowToken,
        address _trustedRelayer
    ) Ownable(msg.sender) {
        borrowToken = IERC20(_borrowToken);
        trustedRelayer = _trustedRelayer;
    }

    /// @notice Borrow ERC-20 tokens against ZEC collateral
    /// @dev Only callable by the trusted relayer
    /// @param borrower Address to receive borrowed tokens
    /// @param collateralZat ZEC collateral amount in zatoshis (attested by relayer)
    /// @param amount ERC-20 tokens to borrow
    /// @param positionId ZLend position identifier
    function borrow(
        address borrower,
        uint256 collateralZat,
        uint256 amount,
        bytes32 positionId
    ) external {
        require(msg.sender == trustedRelayer, "Only relayer");
        require(!loans[positionId].active, "Loan already active");
        require(borrower != address(0), "Zero address");
        require(amount > 0, "Zero amount");
        require(
            collateralZat * 100 >= amount * COLLATERAL_RATIO,
            "Insufficient collateral"
        );

        loans[positionId] = Loan({
            positionId: positionId,
            collateralZat: collateralZat,
            borrowedAmount: amount,
            borrower: borrower,
            active: true
        });

        require(borrowToken.transfer(borrower, amount), "Transfer failed");
        emit Borrowed(positionId, borrower, collateralZat, amount);
    }

    function setRelayer(address _relayer) external onlyOwner {
        emit RelayerUpdated(trustedRelayer, _relayer);
        trustedRelayer = _relayer;
    }

    function getLoan(
        bytes32 positionId
    ) external view returns (Loan memory) {
        return loans[positionId];
    }
}
