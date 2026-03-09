import { Badge } from "../ui/Badge.tsx";
import { VAULT_STATUS, VAULT_STATUS_LABELS, type Vault } from "../../types/vault.ts";

interface ActiveLoansTableProps {
  vaults: Vault[];
}

const STATUS_VARIANT = {
  [VAULT_STATUS.BORROWED]: "danger",
  [VAULT_STATUS.REPAID]: "success",
  [VAULT_STATUS.WITHDRAWN]: "default",
} as const;

export function ActiveLoansTable({ vaults }: ActiveLoansTableProps) {
  if (vaults.length === 0) return null;

  return (
    <div>
      <h3 className="mb-3 text-xs font-medium uppercase tracking-wider text-surface-400">
        Active Loans
      </h3>
      <div className="overflow-x-auto rounded-lg border border-surface-700/50">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-surface-700/50 text-left">
              <th className="px-4 py-2 text-xs font-medium text-surface-400">
                Vault
              </th>
              <th className="px-4 py-2 text-xs font-medium text-surface-400">
                Principal
              </th>
              <th className="px-4 py-2 text-xs font-medium text-surface-400">
                Status
              </th>
            </tr>
          </thead>
          <tbody>
            {vaults.map((v) => (
              <tr
                key={v.id}
                className="border-b border-surface-800/50 last:border-0"
              >
                <td className="px-4 py-2 font-mono text-surface-300">
                  #{v.id.slice(0, 6)}
                </td>
                <td className="px-4 py-2 font-mono text-white">
                  {v.borrowedAmount ?? "—"} USDT
                </td>
                <td className="px-4 py-2">
                  <Badge
                    variant={
                      STATUS_VARIANT[
                        v.status as keyof typeof STATUS_VARIANT
                      ] ?? "default"
                    }
                  >
                    {VAULT_STATUS_LABELS[v.status]}
                  </Badge>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
