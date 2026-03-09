import { useAccount } from "wagmi";
import { Stepper } from "../ui/Stepper.tsx";
import { DepositAddressCard } from "../deposit/DepositAddressCard.tsx";
import { DepositForm } from "../deposit/DepositForm.tsx";
import { VaultList } from "../deposit/VaultList.tsx";
import { VAULT_STATUS } from "../../types/vault.ts";
import type { Vault } from "../../types/vault.ts";

interface DepositViewProps {
  vaults: Vault[];
  onAddVault: (zecAmount: string, shieldedAddress: string) => void;
}

export function DepositView({ vaults, onAddVault }: DepositViewProps) {
  const { isConnected } = useAccount();

  if (!isConnected) {
    return (
      <p className="py-20 text-center text-surface-500">
        Connect your wallet to start depositing ZEC.
      </p>
    );
  }

  return (
    <div className="space-y-8">
      <div className="flex justify-center">
        <Stepper currentStatus={VAULT_STATUS.DEPOSITED} />
      </div>

      <div className="grid gap-4 sm:grid-cols-2">
        <DepositAddressCard />
        <DepositForm onDeposit={onAddVault} />
      </div>

      <VaultList vaults={vaults} />
    </div>
  );
}
