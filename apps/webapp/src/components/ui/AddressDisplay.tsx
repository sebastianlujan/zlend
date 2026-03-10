import { useState } from "react";
import { Copy, Check } from "lucide-react";

interface AddressDisplayProps {
  address: string;
  startChars?: number;
  endChars?: number;
  copyable?: boolean;
  className?: string;
}

export function AddressDisplay({
  address,
  startChars = 8,
  endChars = 8,
  copyable = true,
  className = "",
}: AddressDisplayProps) {
  const [copied, setCopied] = useState(false);

  const truncated =
    address.length <= startChars + endChars + 3
      ? address
      : `${address.slice(0, startChars)}...${address.slice(-endChars)}`;

  const handleCopy = async () => {
    await navigator.clipboard.writeText(address);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <span className={`group relative inline-flex items-center gap-1.5 ${className}`}>
      <span className="font-mono text-xs text-surface-300" title={address}>
        {truncated}
      </span>
      {copyable && (
        <button
          onClick={handleCopy}
          className="shrink-0 cursor-pointer rounded p-0.5 text-surface-400 hover:text-surface-200 transition-colors"
          title="Copy address"
        >
          {copied ? <Check size={12} className="text-green-400" /> : <Copy size={12} />}
        </button>
      )}
    </span>
  );
}
