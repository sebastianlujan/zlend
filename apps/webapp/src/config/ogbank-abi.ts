export const ogBankAbi = [
  // --- Read functions ---
  {
    type: "function",
    name: "OWNER",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "_owner", type: "address" }],
  },
  {
    type: "function",
    name: "VERIFIER",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "_verifier", type: "address" }],
  },
  {
    type: "function",
    name: "AAVE_POOL",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "_pool", type: "address" }],
  },
  {
    type: "function",
    name: "COLLATERAL_TOKEN",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "_token", type: "address" }],
  },
  {
    type: "function",
    name: "BORROW_TOKEN",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "_token", type: "address" }],
  },
  {
    type: "function",
    name: "borrowNullifiers",
    stateMutability: "view",
    inputs: [{ name: "_nullifier", type: "bytes32" }],
    outputs: [{ name: "_exists", type: "bool" }],
  },
  {
    type: "function",
    name: "repayNullifiers",
    stateMutability: "view",
    inputs: [{ name: "_nullifier", type: "bytes32" }],
    outputs: [{ name: "_exists", type: "bool" }],
  },
  {
    type: "function",
    name: "consumedNullifiers",
    stateMutability: "view",
    inputs: [{ name: "_nullifier", type: "bytes32" }],
    outputs: [{ name: "_consumed", type: "bool" }],
  },
  // --- Write functions ---
  {
    type: "function",
    name: "supplyCollateral",
    stateMutability: "nonpayable",
    inputs: [{ name: "_amount", type: "uint256" }],
    outputs: [],
  },
  {
    type: "function",
    name: "borrow",
    stateMutability: "nonpayable",
    inputs: [
      { name: "_proof", type: "bytes" },
      { name: "_publicInputs", type: "bytes32[]" },
      { name: "_amount", type: "uint256" },
    ],
    outputs: [],
  },
  {
    type: "function",
    name: "repay",
    stateMutability: "nonpayable",
    inputs: [
      { name: "_amount", type: "uint256" },
      { name: "_borrowNullifier", type: "bytes32" },
    ],
    outputs: [],
  },
  {
    type: "function",
    name: "withdrawProof",
    stateMutability: "nonpayable",
    inputs: [
      { name: "_proof", type: "bytes" },
      { name: "_publicInputs", type: "bytes32[]" },
      { name: "_amount", type: "uint256" },
    ],
    outputs: [],
  },
  // --- Events ---
  {
    type: "event",
    name: "CollateralSupplied",
    inputs: [
      { name: "_user", type: "address", indexed: true },
      { name: "_amount", type: "uint256", indexed: false },
    ],
  },
  {
    type: "event",
    name: "BorrowExecuted",
    inputs: [
      { name: "_user", type: "address", indexed: true },
      { name: "_amount", type: "uint256", indexed: false },
      { name: "_borrowNullifier", type: "bytes32", indexed: true },
    ],
  },
  {
    type: "event",
    name: "RepayExecuted",
    inputs: [
      { name: "_user", type: "address", indexed: true },
      { name: "_amount", type: "uint256", indexed: false },
      { name: "_borrowNullifier", type: "bytes32", indexed: true },
    ],
  },
  {
    type: "event",
    name: "FinishPayment",
    inputs: [
      { name: "_ogbank", type: "address", indexed: true },
      { name: "_amount", type: "uint256", indexed: false },
      { name: "_recipient", type: "address", indexed: false },
      { name: "_originAddress", type: "address", indexed: false },
    ],
  },
  // --- Errors ---
  { type: "error", name: "OGBank_InvalidProof", inputs: [] },
  { type: "error", name: "OGBank_NullifierAlreadyUsed", inputs: [] },
  { type: "error", name: "OGBank_BorrowNullifierNotFound", inputs: [] },
  { type: "error", name: "OGBank_NotRepaid", inputs: [] },
  { type: "error", name: "OGBank_AlreadyConsumed", inputs: [] },
  { type: "error", name: "OGBank_ZeroAmount", inputs: [] },
  { type: "error", name: "OGBank_OnlyOwner", inputs: [] },
] as const;

export const erc20ApproveAbi = [
  {
    type: "function",
    name: "approve",
    stateMutability: "nonpayable",
    inputs: [
      { name: "spender", type: "address" },
      { name: "amount", type: "uint256" },
    ],
    outputs: [{ name: "", type: "bool" }],
  },
  {
    type: "function",
    name: "allowance",
    stateMutability: "view",
    inputs: [
      { name: "owner", type: "address" },
      { name: "spender", type: "address" },
    ],
    outputs: [{ name: "", type: "uint256" }],
  },
] as const;
