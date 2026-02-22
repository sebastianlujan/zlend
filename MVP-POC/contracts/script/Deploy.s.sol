// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Script, console} from "forge-std/Script.sol";
import {MockUSDC} from "../src/MockUSDC.sol";
import {ZLendMVP} from "../src/ZLendMVP.sol";

contract DeployScript is Script {
    function run() external {
        uint256 deployerKey = vm.envUint("DEPLOYER_PRIVATE_KEY");
        address relayer = vm.envAddress("RELAYER_ADDRESS");

        vm.startBroadcast(deployerKey);

        // 1. Deploy MockUSDC with 1M mUSDC (6 decimals)
        uint256 initialSupply = 1_000_000 * 1e6;
        MockUSDC usdc = new MockUSDC(initialSupply);
        console.log("MockUSDC deployed at:", address(usdc));

        // 2. Deploy ZLendMVP with MockUSDC + relayer address
        ZLendMVP zlend = new ZLendMVP(address(usdc), relayer);
        console.log("ZLendMVP deployed at:", address(zlend));

        // 3. Transfer MockUSDC supply to ZLendMVP (so it can lend)
        usdc.transfer(address(zlend), initialSupply);
        console.log("Transferred", initialSupply, "mUSDC to ZLendMVP");

        vm.stopBroadcast();
    }
}
