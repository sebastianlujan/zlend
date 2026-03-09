import { useAccount } from "wagmi";
import { VAULT_STATUS, type Vault } from "../../types/vault.ts";

interface SummaryStripProps {
  vaults: Vault[];
}

export function SummaryStrip({ vaults }: SummaryStripProps) {
  const { chain } = useAccount();

  const totalZec = vaults
    .filter((v) => v.status !== VAULT_STATUS.WITHDRAWN)
    .reduce((sum, v) => sum + parseFloat(v.zecAmount || "0"), 0);

  const totalBorrowed = vaults
    .filter((v) => v.status === VAULT_STATUS.BORROWED)
    .reduce((sum, v) => sum + parseFloat(v.borrowedAmount || "0"), 0);

  const activeLoans = vaults.filter(
    (v) => v.status === VAULT_STATUS.BORROWED,
  ).length;

  return (
    <div className="flex flex-wrap items-center gap-6 rounded-lg border border-surface-800/50 bg-surface-900/30 px-5 py-3 text-sm">
      <StatItem label="ZEC Deposited" value={totalZec.toFixed(2)} />
      <Separator />
      <StatItem label="USDT Borrowed" value={totalBorrowed.toFixed(2)} />
      <Separator />
      <StatItem label="Active Loans" value={String(activeLoans)} />
      <Separator />
      <StatItem label="Network" value={chain?.name ?? "—"} />
    </div>
  );
}

function StatItem({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <span className="text-xs text-surface-500">{label}</span>
      <span className="ml-2 font-mono text-sm font-medium text-surface-200">
        {value}
      </span>
    </div>
  );
}

function Separator() {
  return <div className="h-4 w-px bg-surface-700" />;
}
