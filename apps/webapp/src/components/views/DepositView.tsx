import { useState } from "react";
import { useAccount } from "wagmi";
import { DepositCard } from "../deposit/DepositCard.tsx";
import { VaultList } from "../deposit/VaultList.tsx";
import { GenerateUSDTCard } from "../borrow/GenerateUSDTCard.tsx";
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
  const [autoOpenVaults, setAutoOpenVaults] = useState(false);
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
        <GenerateUSDTCard
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
          onWithdrawn={() =>
            onUpdateVault(selectedVault.id, {
              status: VAULT_STATUS.WITHDRAWN,
            })
          }
        />
      )}
    </>
  );

  if (selectedVault) {
    return (
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-5">
        <div className="space-y-6 lg:col-span-3">
          <DepositCard
            onDeposit={(amount, addr) => {
              const id = onAddVault(amount, addr);
              setSelectedVaultId(id);
              setAutoOpenVaults(true);
              return id;
            }}
          />
          <VaultList
            vaults={vaults}
            selectedVaultId={selectedVaultId}
            onSelectVault={setSelectedVaultId}
            autoOpen={autoOpenVaults}
          />
        </div>

        <div className="space-y-4 lg:col-span-2 lg:sticky lg:top-24 lg:self-start">
          <Stepper currentStatus={selectedVault.status} />
          {actionCard}
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-8">
      <DepositCard
        onDeposit={(amount, addr) => {
          const id = onAddVault(amount, addr);
          setSelectedVaultId(id);
          setAutoOpenVaults(true);
          return id;
        }}
      />
      <VaultList
        vaults={vaults}
        selectedVaultId={selectedVaultId}
        onSelectVault={setSelectedVaultId}
        autoOpen={autoOpenVaults}
      />
    </div>
  );
}
