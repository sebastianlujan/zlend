import { useAccount } from "wagmi";

export function AccountInfo() {
  const { address, chain } = useAccount();

  if (!address) return null;

  return (
    <div className="flex items-center gap-4 rounded-lg border border-surface-700/50 bg-surface-900/50 px-4 py-3">
      <div>
        <p className="text-xs text-surface-500">Connected</p>
        <p className="font-mono text-sm text-surface-100">
          {address.slice(0, 6)}...{address.slice(-4)}
        </p>
      </div>
      <div className="border-l border-surface-700 pl-4">
        <p className="text-xs text-surface-500">Network</p>
        <p className="font-mono text-sm text-surface-100">
          {chain?.name ?? "Unknown"}
        </p>
      </div>
    </div>
  );
}
