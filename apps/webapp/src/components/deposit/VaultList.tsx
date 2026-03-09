import { VaultCard } from "./VaultCard.tsx";
import type { Vault } from "../../types/vault.ts";

interface VaultListProps {
  vaults: Vault[];
}

export function VaultList({ vaults }: VaultListProps) {
  if (vaults.length === 0) return null;

  return (
    <div>
      <h3 className="mb-3 text-xs font-medium uppercase tracking-wider text-surface-400">
        Your Vaults
      </h3>
      <div className="space-y-2">
        {vaults.map((vault) => (
          <VaultCard key={vault.id} vault={vault} />
        ))}
      </div>
    </div>
  );
}
