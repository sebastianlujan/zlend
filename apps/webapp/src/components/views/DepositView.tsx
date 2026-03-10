import { useState } from "react";
import { useAccount } from "wagmi";
import { DepositCard } from "../deposit/DepositCard.tsx";
import { VaultList } from "../deposit/VaultList.tsx";
import { GenerateAUSDCard } from "../borrow/GenerateAUSDCard.tsx";
import { RepayCard } from "../repay/RepayCard.tsx";
import { WithdrawCard } from "../repay/WithdrawCard.tsx";
import { Stepper } from "../ui/Stepper.tsx";
import { VAULT_STATUS } from "../../types/vault.ts";
import type { Vault } from "../../types/vault.ts";

interface VaultsViewProps {
  vaults: Vault[];
  onAddVault: (zecAmount: string, shieldedAddress: string) => string;
  onUpdateVault: (id: string, updates: Partial<Vault>) => void;
}

export function DepositView({ vaults, onAddVault, onUpdateVault }: VaultsViewProps) {
  const { isConnected } = useAccount();

  const [selectedVaultId, setSelectedVaultId] = useState<string>("");
  const selectedVault = vaults.find((v) => v.id === selectedVaultId);

  if (!isConnected) {
    return (
      <p className="py-20 text-center text-surface-500">
        Connect your wallet to start depositing ZEC.
      </p>
    );
  }

  const actionCard = selectedVault && (
    <>
      {(selectedVault.status === VAULT_STATUS.DEPOSITED ||
        selectedVault.status === VAULT_STATUS.PROOF_READY) && (
        <GenerateAUSDCard
          vault={selectedVault}
          onProofGenerated={(proofData) =>
            onUpdateVault(selectedVault.id, {
              status: VAULT_STATUS.PROOF_READY,
              proofData,
            })
          }
          onBorrowed={(borrowNullifier, amount) =>
            onUpdateVault(selectedVault.id, {
              status: VAULT_STATUS.BORROWED,
              borrowNullifier,
              borrowedAmount: amount,
            })
          }
        />
      )}

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
          onWithdrawn={() => {
            onUpdateVault(selectedVault.id, {
              status: VAULT_STATUS.WITHDRAWN,
            });
            setSelectedVaultId("");
          }}
        />
      )}
    </>
  );

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <DepositCard
          onDeposit={(amount, addr) => {
            const id = onAddVault(amount, addr);
            setSelectedVaultId(id);
            return id;
          }}
        />

        {selectedVault ? (
          <div className="space-y-4 lg:sticky lg:top-24 lg:self-start">
            <Stepper currentStatus={selectedVault.status} />
            {actionCard}
          </div>
        ) : (
          <div className="hidden lg:flex flex-col items-center justify-center rounded-2xl border border-dashed border-surface-700 bg-surface-900/40 p-8 text-center">
            <svg
              className="mb-4 h-12 w-12 text-surface-600"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              strokeWidth={1.5}
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M20.25 7.5l-.625 10.632a2.25 2.25 0 01-2.247 2.118H6.622a2.25 2.25 0 01-2.247-2.118L3.75 7.5m8.25 3v6.75m0 0l-3-3m3 3l3-3M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125z"
              />
            </svg>
            <p className="text-sm font-medium text-surface-400">
              No vault selected
            </p>
            <p className="mt-1 text-xs text-surface-600">
              Deposit ZEC to create a new vault, or select an existing one below to continue the flow.
            </p>
          </div>
        )}
      </div>

      <VaultList
        vaults={vaults}
        selectedVaultId={selectedVaultId}
        onSelectVault={setSelectedVaultId}
      />
    </div>
  );
}
