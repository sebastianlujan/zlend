import { useState } from "react";
import { useAccount } from "wagmi";
import { Stepper } from "../ui/Stepper.tsx";
import { RepayCard } from "../repay/RepayCard.tsx";
import { WithdrawCard } from "../repay/WithdrawCard.tsx";
import { VAULT_STATUS } from "../../types/vault.ts";
import type { Vault } from "../../types/vault.ts";
import type { TabKey } from "../ui/TabNav.tsx";

interface RepayWithdrawViewProps {
  vaults: Vault[];
  onUpdateVault: (id: string, updates: Partial<Vault>) => void;
  onTabChange: (tab: TabKey) => void;
}

export function RepayWithdrawView({
  vaults,
  onUpdateVault,
  onTabChange,
}: RepayWithdrawViewProps) {
  const { isConnected } = useAccount();

  const actionableVaults = vaults.filter(
    (v) => v.status === VAULT_STATUS.BORROWED || v.status === VAULT_STATUS.REPAID,
  );

  const [selectedVaultId, setSelectedVaultId] = useState<string>("");
  const selectedVault = actionableVaults.find((v) => v.id === selectedVaultId);

  if (!isConnected) {
    return (
      <p className="py-20 text-center text-surface-500">
        Connect your wallet to manage loans.
      </p>
    );
  }

  if (actionableVaults.length === 0) {
    return (
      <div className="py-20 text-center">
        <p className="text-surface-500">
          No active loans. Borrow first to start repaying.
        </p>
        <button
          onClick={() => onTabChange("borrow")}
          className="mt-4 text-sm font-medium text-primary-400 hover:text-primary-300 cursor-pointer"
        >
          Go to Borrow →
        </button>
      </div>
    );
  }

  const currentStatus = selectedVault?.status ?? VAULT_STATUS.BORROWED;

  return (
    <div className="space-y-8">
      <div className="flex justify-center">
        <Stepper currentStatus={currentStatus} />
      </div>

      <div>
        <label className="mb-2 block text-xs font-medium uppercase tracking-wider text-surface-400">
          Select Loan
        </label>
        <select
          value={selectedVaultId}
          onChange={(e) => setSelectedVaultId(e.target.value)}
          className="w-full rounded-lg border border-surface-700/50 bg-surface-800/50 px-4 py-3 font-mono text-sm text-surface-100 focus:border-primary-500 focus:outline-none"
        >
          <option value="">Choose a loan...</option>
          {actionableVaults.map((v) => (
            <option key={v.id} value={v.id}>
              Vault #{v.id.slice(0, 6)} — {v.borrowedAmount ?? "?"} USDT
              {v.status === VAULT_STATUS.REPAID ? " (Repaid)" : ""}
            </option>
          ))}
        </select>
      </div>

      {selectedVault && (
        <>
          {selectedVault.status === VAULT_STATUS.BORROWED && (
            <RepayCard
              vault={selectedVault}
              onRepaid={() =>
                onUpdateVault(selectedVault.id, {
                  status: VAULT_STATUS.REPAID,
                })
              }
            />
          )}

          {selectedVault.status === VAULT_STATUS.REPAID && (
            <WithdrawCard
              vault={selectedVault}
              onWithdrawn={() =>
                onUpdateVault(selectedVault.id, {
                  status: VAULT_STATUS.WITHDRAWN,
                })
              }
            />
          )}
        </>
      )}
    </div>
  );
}
