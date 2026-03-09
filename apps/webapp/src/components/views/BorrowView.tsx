import { useState } from "react";
import { useAccount } from "wagmi";
import { Stepper } from "../ui/Stepper.tsx";
import { ProofGenerationCard } from "../borrow/ProofGenerationCard.tsx";
import { BorrowForm } from "../borrow/BorrowForm.tsx";
import { ActiveLoansTable } from "../borrow/ActiveLoansTable.tsx";
import { VAULT_STATUS } from "../../types/vault.ts";
import type { Vault } from "../../types/vault.ts";
import type { TabKey } from "../ui/TabNav.tsx";

interface BorrowViewProps {
  vaults: Vault[];
  onUpdateVault: (id: string, updates: Partial<Vault>) => void;
  onTabChange: (tab: TabKey) => void;
}

export function BorrowView({ vaults, onUpdateVault, onTabChange }: BorrowViewProps) {
  const { isConnected } = useAccount();

  const eligibleVaults = vaults.filter(
    (v) => v.status === VAULT_STATUS.DEPOSITED || v.status === VAULT_STATUS.PROOF_READY,
  );
  const borrowedVaults = vaults.filter(
    (v) =>
      v.status === VAULT_STATUS.BORROWED ||
      v.status === VAULT_STATUS.REPAID ||
      v.status === VAULT_STATUS.WITHDRAWN,
  );

  const [selectedVaultId, setSelectedVaultId] = useState<string>("");
  const selectedVault = eligibleVaults.find((v) => v.id === selectedVaultId);

  if (!isConnected) {
    return (
      <p className="py-20 text-center text-surface-500">
        Connect your wallet to borrow.
      </p>
    );
  }

  if (eligibleVaults.length === 0 && borrowedVaults.length === 0) {
    return (
      <div className="py-20 text-center">
        <p className="text-surface-500">
          No vaults available. Deposit ZEC first to start borrowing.
        </p>
        <button
          onClick={() => onTabChange("deposit")}
          className="mt-4 text-sm font-medium text-primary-400 hover:text-primary-300 cursor-pointer"
        >
          Go to Deposit →
        </button>
      </div>
    );
  }

  const currentStatus = selectedVault?.status ?? VAULT_STATUS.PROOF_READY;

  return (
    <div className="space-y-8">
      <div className="flex justify-center">
        <Stepper currentStatus={currentStatus} />
      </div>

      {eligibleVaults.length > 0 && (
        <>
          <div>
            <label className="mb-2 block text-xs font-medium uppercase tracking-wider text-surface-400">
              Select Vault
            </label>
            <select
              value={selectedVaultId}
              onChange={(e) => setSelectedVaultId(e.target.value)}
              className="w-full rounded-lg border border-surface-700/50 bg-surface-800/50 px-4 py-3 font-mono text-sm text-surface-100 focus:border-primary-500 focus:outline-none"
            >
              <option value="">Choose a vault...</option>
              {eligibleVaults.map((v) => (
                <option key={v.id} value={v.id}>
                  Vault #{v.id.slice(0, 6)} — {v.zecAmount} ZEC
                </option>
              ))}
            </select>
          </div>

          {selectedVault && (
            <>
              <ProofGenerationCard
                vault={selectedVault}
                onProofGenerated={(proofData) =>
                  onUpdateVault(selectedVault.id, {
                    status: VAULT_STATUS.PROOF_READY,
                    proofData,
                  })
                }
              />
              <BorrowForm
                vault={selectedVault}
                onBorrowed={(borrowNullifier, amount) =>
                  onUpdateVault(selectedVault.id, {
                    status: VAULT_STATUS.BORROWED,
                    borrowNullifier,
                    borrowedAmount: amount,
                  })
                }
              />
            </>
          )}
        </>
      )}

      <ActiveLoansTable vaults={borrowedVaults} />
    </div>
  );
}
