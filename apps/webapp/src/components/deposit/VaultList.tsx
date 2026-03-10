import { useMemo } from "react";
import { Vault as VaultIcon } from "lucide-react";
import { VaultCard } from "./VaultCard.tsx";
import { EmptyState } from "../ui/EmptyState.tsx";
import { usePagination } from "../../hooks/usePagination.ts";
import { VAULT_STATUS, STATUS_ORDER, type Vault } from "../../types/vault.ts";

interface VaultListProps {
  vaults: Vault[];
  selectedVaultId?: string;
  onSelectVault?: (id: string) => void;
}

export function VaultList({ vaults, selectedVaultId, onSelectVault }: VaultListProps) {
  const sortedVaults = useMemo(
    () => [...vaults].sort((a, b) => (STATUS_ORDER[a.status] ?? 9) - (STATUS_ORDER[b.status] ?? 9)),
    [vaults],
  );

  const { pageItems, page, totalPages, hasPrev, hasNext, prev, next } =
    usePagination(sortedVaults, 10);

  const selectedVault = selectedVaultId
    ? vaults.find((v) => v.id === selectedVaultId)
    : null;

  const isSelectable = (vault: Vault) =>
    vault.status !== VAULT_STATUS.WITHDRAWN;

  if (vaults.length === 0) {
    return (
      <EmptyState
        icon={VaultIcon}
        heading="No vaults yet"
        description="Deposit ZEC above to create your first vault and start borrowing."
      />
    );
  }

  return (
    <div>
      <div className="flex items-center justify-between rounded-lg border border-surface-700/50 bg-surface-900 px-5 py-3.5">
        <div className="flex items-center gap-3">
          <span className="text-sm font-medium text-white">My Vaults</span>
          {selectedVault && (
            <span className="font-mono text-xs text-primary-400">
              #{selectedVault.id.slice(0, 6)} selected
            </span>
          )}
        </div>
        <span className="rounded-full bg-surface-700 px-2 py-0.5 font-mono text-[10px] text-surface-300">
          {vaults.length}
        </span>
      </div>

      <div className="space-y-2 pt-3">
        {pageItems.map((vault) => (
          <VaultCard
            key={vault.id}
            vault={vault}
            selected={vault.id === selectedVaultId}
            selectable={isSelectable(vault)}
            onSelect={onSelectVault}
          />
        ))}
      </div>

      {totalPages > 1 && (
        <div className="mt-3 flex items-center justify-center gap-3">
          <button
            onClick={prev}
            disabled={!hasPrev}
            className="rounded px-2 py-1 text-xs text-surface-500 hover:bg-surface-800 disabled:cursor-not-allowed disabled:opacity-30"
          >
            Prev
          </button>
          <span className="font-mono text-xs text-surface-500">
            {page + 1} / {totalPages}
          </span>
          <button
            onClick={next}
            disabled={!hasNext}
            className="rounded px-2 py-1 text-xs text-surface-500 hover:bg-surface-800 disabled:cursor-not-allowed disabled:opacity-30"
          >
            Next
          </button>
        </div>
      )}
    </div>
  );
}
