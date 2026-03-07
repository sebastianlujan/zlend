import { useAccount, useReadContracts } from "wagmi";
import { formatUnits, type Address } from "viem";
import { erc20Abi } from "../../config/contracts.ts";
import { Card } from "../ui/Card.tsx";

interface TokenBalanceProps {
  label: string;
  tokenAddress: Address;
}

export function TokenBalance({ label, tokenAddress }: TokenBalanceProps) {
  const { address } = useAccount();

  const { data, isLoading } = useReadContracts({
    contracts: [
      {
        address: tokenAddress,
        abi: erc20Abi,
        functionName: "symbol",
      },
      {
        address: tokenAddress,
        abi: erc20Abi,
        functionName: "decimals",
      },
      {
        address: tokenAddress,
        abi: erc20Abi,
        functionName: "balanceOf",
        args: [address!],
      },
    ],
    query: { enabled: !!address },
  });

  const symbol = data?.[0]?.result as string | undefined;
  const decimals = data?.[1]?.result as number | undefined;
  const balance = data?.[2]?.result as bigint | undefined;

  const formatted =
    balance !== undefined && decimals !== undefined
      ? parseFloat(formatUnits(balance, decimals)).toFixed(4)
      : "--";

  return (
    <Card>
      <p className="text-xs font-medium uppercase tracking-wider text-surface-400">
        {label}{symbol ? ` (${symbol})` : ""}
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
              {symbol ?? ""}
            </span>
          </>
        )}
      </div>
    </Card>
  );
}
