import { Badge } from "../ui/Badge.tsx";
import { VAULT_STATUS, VAULT_STATUS_LABELS, type Vault } from "../../types/vault.ts";

interface VaultCardProps {
  vault: Vault;
}

const STATUS_VARIANT = {
  [VAULT_STATUS.DEPOSITED]: "info",
  [VAULT_STATUS.PROOF_READY]: "warning",
  [VAULT_STATUS.BORROWED]: "danger",
  [VAULT_STATUS.REPAID]: "success",
  [VAULT_STATUS.WITHDRAWN]: "default",
} as const;

function timeAgo(ts: number): string {
  const diff = Date.now() - ts;
  const mins = Math.floor(diff / 60_000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

export function VaultCard({ vault }: VaultCardProps) {
  return (
    <div className="flex items-center justify-between rounded-lg border border-surface-700/50 bg-surface-800/30 px-4 py-3">
      <div className="flex items-center gap-4">
        <span className="font-mono text-sm text-surface-300">
          #{vault.id.slice(0, 6)}
        </span>
        <span className="font-mono text-sm font-medium text-white">
          {vault.zecAmount} ZEC
        </span>
        <Badge variant={STATUS_VARIANT[vault.status]}>
          {VAULT_STATUS_LABELS[vault.status]}
        </Badge>
      </div>
      <span className="text-xs text-surface-500">
        {timeAgo(vault.createdAt)}
      </span>
    </div>
  );
}
