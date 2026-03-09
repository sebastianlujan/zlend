import type { Hex } from "viem";

export const VAULT_STATUS = {
  DEPOSITED: "deposited",
  PROOF_READY: "proof_ready",
  BORROWED: "borrowed",
  REPAID: "repaid",
  WITHDRAWN: "withdrawn",
} as const;

export type VaultStatus = (typeof VAULT_STATUS)[keyof typeof VAULT_STATUS];

export interface Vault {
  id: string;
  zecAmount: string;
  shieldedAddress: string;
  userSecret: string;
  nonce: string;
  status: VaultStatus;
  borrowNullifier?: Hex;
  proofData?: { proof: Hex; publicInputs: Hex[] };
  borrowedAmount?: string;
  createdAt: number;
}

export const VAULT_STATUS_LABELS: Record<VaultStatus, string> = {
  [VAULT_STATUS.DEPOSITED]: "Deposited",
  [VAULT_STATUS.PROOF_READY]: "Proof Ready",
  [VAULT_STATUS.BORROWED]: "Borrowed",
  [VAULT_STATUS.REPAID]: "Repaid",
  [VAULT_STATUS.WITHDRAWN]: "Withdrawn",
};

export const LIFECYCLE_STEPS = [
  { key: VAULT_STATUS.DEPOSITED, label: "Deposit" },
  { key: VAULT_STATUS.PROOF_READY, label: "Proof" },
  { key: VAULT_STATUS.BORROWED, label: "Borrow" },
  { key: VAULT_STATUS.REPAID, label: "Repay" },
  { key: VAULT_STATUS.WITHDRAWN, label: "Withdraw" },
] as const;
