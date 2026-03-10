import { Coins, DollarSign, Box } from "lucide-react";
import { VAULT_STATUS, type Vault } from "../../types/vault.ts";
import type { LucideIcon } from "lucide-react";

interface SummaryStripProps {
  vaults: Vault[];
}

export function SummaryStrip({ vaults }: SummaryStripProps) {
  const totalZec = vaults
    .filter((v) => v.status !== VAULT_STATUS.WITHDRAWN)
    .reduce((sum, v) => sum + parseFloat(v.zecAmount || "0"), 0);

  const totalBorrowed = vaults
    .filter((v) => v.status === VAULT_STATUS.BORROWED)
    .reduce((sum, v) => sum + parseFloat(v.borrowedAmount || "0"), 0);

  const totalVaults = vaults.filter(
    (v) => v.status !== VAULT_STATUS.WITHDRAWN,
  ).length;

  return (
    <div className="grid grid-cols-2 gap-4 rounded-lg border border-surface-700/50 bg-surface-900/50 backdrop-blur-sm px-5 py-3 text-sm sm:flex sm:flex-wrap sm:items-center sm:gap-6">
      <StatItem icon={Coins} label="ZEC Deposited" value={totalZec.toFixed(2)} valueClass="glow-gold-text text-amber-300" />
      <StatItem icon={DollarSign} label="AUSD Generated" value={totalBorrowed.toFixed(2)} valueClass="text-emerald-400" />
      <StatItem icon={Box} label="Vaults" value={String(totalVaults)} valueClass="text-primary-400" />
    </div>
  );
}

function StatItem({ icon: Icon, label, value, valueClass = "text-surface-200" }: { icon: LucideIcon; label: string; value: string; valueClass?: string }) {
  return (
    <div className="flex items-center gap-2">
      <Icon size={14} className="text-surface-500 shrink-0 hidden sm:block" />
      <div>
        <span className="text-xs text-surface-500">{label}</span>
        <span className={`ml-2 font-mono text-sm font-medium tabular-nums ${valueClass}`}>
          {value}
        </span>
      </div>
    </div>
  );
}
