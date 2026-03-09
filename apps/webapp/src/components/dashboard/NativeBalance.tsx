import { useAccount, useBalance } from "wagmi";
import { formatUnits } from "viem";
import { Card } from "../ui/Card.tsx";

export function NativeBalance() {
  const { address } = useAccount();
  const { data: balance, isLoading } = useBalance({ address });

  const formatted =
    balance !== undefined
      ? parseFloat(formatUnits(balance.value, balance.decimals)).toFixed(4)
      : "--";

  return (
    <Card>
      <p className="text-xs font-medium uppercase tracking-wider text-surface-400">
        Native Token{balance?.symbol ? ` (${balance.symbol})` : ""}
      </p>
      <div className="mt-3 flex items-baseline gap-2">
        {isLoading ? (
          <span className="text-2xl font-bold text-surface-500">...</span>
        ) : !address ? (
          <span className="text-sm text-surface-500">
            Connect wallet to view
          </span>
        ) : (
          <>
            <span className="font-mono text-2xl font-bold text-white">
              {formatted}
            </span>
            <span className="font-mono text-sm text-surface-400">
              {balance?.symbol ?? ""}
            </span>
          </>
        )}
      </div>
    </Card>
  );
}
