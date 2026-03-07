import type { Address } from "viem";

export const addresses = {
  ogBank: import.meta.env.VITE_OGBANK_ADDRESS as Address | undefined,
  collateralToken: import.meta.env.VITE_COLLATERAL_TOKEN_ADDRESS as Address,
  borrowToken: import.meta.env.VITE_BORROW_TOKEN_ADDRESS as Address,
  aavePool: import.meta.env.VITE_AAVE_POOL_ADDRESS as Address,
  verifier: import.meta.env.VITE_VERIFIER_ADDRESS as Address | undefined,
} as const;

export const erc20Abi = [
  {
    type: "function",
    name: "balanceOf",
    stateMutability: "view",
    inputs: [{ name: "account", type: "address" }],
    outputs: [{ name: "", type: "uint256" }],
  },
  {
    type: "function",
    name: "symbol",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "string" }],
  },
  {
    type: "function",
    name: "decimals",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "uint8" }],
  },
] as const;
