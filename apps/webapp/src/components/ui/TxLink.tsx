import { ExternalLink } from "lucide-react";
import { useAccount } from "wagmi";
import type { Hex } from "viem";

interface TxLinkProps {
  hash: Hex;
  label?: string;
  className?: string;
}

export function TxLink({ hash, label, className = "" }: TxLinkProps) {
  const { chain } = useAccount();
  const explorerUrl = chain?.blockExplorers?.default?.url;

  const truncated = `${hash.slice(0, 6)}...${hash.slice(-4)}`;
  const href = explorerUrl ? `${explorerUrl}/tx/${hash}` : undefined;

  if (!href) {
    return (
      <span className={`inline-flex items-center gap-1 font-mono text-xs text-surface-400 ${className}`}>
        {label ?? truncated}
      </span>
    );
  }

  return (
    <a
      href={href}
      target="_blank"
      rel="noopener noreferrer"
      className={`inline-flex items-center gap-1 font-mono text-xs text-accent-400 hover:text-accent-300 transition-colors ${className}`}
    >
      {label ?? truncated}
      <ExternalLink size={11} />
    </a>
  );
}
