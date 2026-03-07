import { useAccount, useBalance } from "wagmi";
import { formatUnits } from "viem";

export function AccountInfo() {
  const { address } = useAccount();
  const { data: balance } = useBalance({ address });

  if (!address) return null;

  return (
    <div className="flex items-center gap-4 rounded-lg border border-surface-700/50 bg-surface-800/50 px-4 py-3">
      <div>
        <p className="text-xs text-surface-400">Connected</p>
        <p className="font-mono text-sm text-surface-100">
          {address.slice(0, 6)}...{address.slice(-4)}
        </p>
      </div>
      {balance && (
        <div className="border-l border-surface-700 pl-4">
          <p className="text-xs text-surface-400">Native Balance</p>
          <p className="font-mono text-sm text-surface-100">
            {parseFloat(formatUnits(balance.value, balance.decimals)).toFixed(4)}{" "}
            {balance.symbol}
          </p>
        </div>
      )}
    </div>
  );
}
