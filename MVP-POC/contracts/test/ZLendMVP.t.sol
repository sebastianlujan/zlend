// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test, console} from "forge-std/Test.sol";
import {MockUSDC} from "../src/MockUSDC.sol";
import {ZLendMVP} from "../src/ZLendMVP.sol";

contract ZLendMVPTest is Test {
    // Re-declare event for expectEmit
    event Borrowed(
        bytes32 indexed positionId,
        address indexed borrower,
        uint256 collateralZat,
        uint256 amount
    );

    MockUSDC public usdc;
    ZLendMVP public zlend;

    address public relayer = makeAddr("relayer");
    address public borrower = makeAddr("borrower");
    address public randomUser = makeAddr("random");

    uint256 constant INITIAL_SUPPLY = 1_000_000 * 1e6; // 1M mUSDC

    function setUp() public {
        // Deploy MockUSDC
        usdc = new MockUSDC(INITIAL_SUPPLY);

        // Deploy ZLendMVP with relayer
        zlend = new ZLendMVP(address(usdc), relayer);

        // Fund the contract with mUSDC
        usdc.transfer(address(zlend), INITIAL_SUPPLY);
    }

    function test_BorrowSucceeds() public {
        bytes32 positionId = bytes32("position-001");
        uint256 collateralZat = 200_000_000; // 2 ZEC worth
        uint256 borrowAmount = 100_000_000; // 100 mUSDC (well within 150%)

        vm.prank(relayer);
        zlend.borrow(borrower, collateralZat, borrowAmount, positionId);

        // Verify loan was created
        ZLendMVP.Loan memory loan = zlend.getLoan(positionId);
        assertEq(loan.borrower, borrower);
        assertEq(loan.collateralZat, collateralZat);
        assertEq(loan.borrowedAmount, borrowAmount);
        assertTrue(loan.active);

        // Verify borrower received tokens
        assertEq(usdc.balanceOf(borrower), borrowAmount);
    }

    function test_BorrowEmitsEvent() public {
        bytes32 positionId = bytes32("position-002");
        uint256 collateralZat = 300_000_000;
        uint256 borrowAmount = 100_000_000;

        vm.expectEmit(true, true, false, true);
        emit Borrowed(positionId, borrower, collateralZat, borrowAmount);

        vm.prank(relayer);
        zlend.borrow(borrower, collateralZat, borrowAmount, positionId);
    }

    function test_RevertWhen_NotRelayer() public {
        bytes32 positionId = bytes32("position-003");

        vm.prank(randomUser);
        vm.expectRevert("Only relayer");
        zlend.borrow(borrower, 200_000_000, 100_000_000, positionId);
    }

    function test_RevertWhen_InsufficientCollateral() public {
        bytes32 positionId = bytes32("position-004");
        uint256 collateralZat = 100_000_000; // 1 ZEC — not enough for 150% ratio
        uint256 borrowAmount = 100_000_000; // needs 1.5 ZEC

        vm.prank(relayer);
        vm.expectRevert("Insufficient collateral");
        zlend.borrow(borrower, collateralZat, borrowAmount, positionId);
    }

    function test_RevertWhen_LoanAlreadyActive() public {
        bytes32 positionId = bytes32("position-005");

        vm.prank(relayer);
        zlend.borrow(borrower, 200_000_000, 100_000_000, positionId);

        // Try to borrow again with same positionId
        vm.prank(relayer);
        vm.expectRevert("Loan already active");
        zlend.borrow(borrower, 200_000_000, 100_000_000, positionId);
    }

    function test_RevertWhen_ZeroAddress() public {
        bytes32 positionId = bytes32("position-006");

        vm.prank(relayer);
        vm.expectRevert("Zero address");
        zlend.borrow(address(0), 200_000_000, 100_000_000, positionId);
    }

    function test_RevertWhen_ZeroAmount() public {
        bytes32 positionId = bytes32("position-007");

        vm.prank(relayer);
        vm.expectRevert("Zero amount");
        zlend.borrow(borrower, 200_000_000, 0, positionId);
    }

    function test_SetRelayer() public {
        address newRelayer = makeAddr("newRelayer");

        zlend.setRelayer(newRelayer);
        assertEq(zlend.trustedRelayer(), newRelayer);
    }

    function test_RevertWhen_NonOwnerSetsRelayer() public {
        vm.prank(randomUser);
        vm.expectRevert();
        zlend.setRelayer(randomUser);
    }

    function test_CollateralRatioBoundary() public {
        bytes32 positionId = bytes32("position-008");

        // Exactly 150%: 150 collateral, 100 borrow → 150*100 >= 100*150 → passes
        vm.prank(relayer);
        zlend.borrow(borrower, 150, 100, positionId);

        // Verify it succeeded
        ZLendMVP.Loan memory loan = zlend.getLoan(positionId);
        assertTrue(loan.active);
    }

    function test_CollateralRatioJustBelow() public {
        bytes32 positionId = bytes32("position-009");

        // Just below 150%: 149 collateral, 100 borrow → 149*100 < 100*150 → fails
        vm.prank(relayer);
        vm.expectRevert("Insufficient collateral");
        zlend.borrow(borrower, 149, 100, positionId);
    }

    function test_MockUSDCDecimals() public view {
        assertEq(usdc.decimals(), 6);
    }

    function test_MockUSDCName() public view {
        assertEq(usdc.name(), "Mock USDC");
        assertEq(usdc.symbol(), "mUSDC");
    }
}
