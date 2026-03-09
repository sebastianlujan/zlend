import { useState } from "react";
import { Card } from "../ui/Card.tsx";
import { Badge } from "../ui/Badge.tsx";
import { Button } from "../ui/Button.tsx";
import { Spinner } from "../ui/Spinner.tsx";

export function DepositAddressCard() {
  const [address, setAddress] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const [copied, setCopied] = useState(false);

  const generateAddress = async () => {
    setLoading(true);
    await new Promise((r) => setTimeout(r, 1500));
    const chars = "abcdefghijklmnopqrstuvwxyz0123456789";
    let addr = "zs1";
    for (let i = 0; i < 60; i++) {
      addr += chars[Math.floor(Math.random() * chars.length)];
    }
    setAddress(addr);
    setLoading(false);
  };

  const copyAddress = async () => {
    if (!address) return;
    await navigator.clipboard.writeText(address);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <Card className="border-dashed border-surface-600">
      <div className="flex items-center gap-2">
        <p className="text-xs font-medium uppercase tracking-wider text-surface-400">
          Deposit Address
        </p>
        <Badge>Simulated</Badge>
      </div>

      {address ? (
        <div className="mt-4">
          <div className="rounded-lg bg-surface-800/80 px-3 py-2">
            <p className="break-all font-mono text-xs text-surface-200">
              {address}
            </p>
          </div>
          <button
            onClick={copyAddress}
            className="mt-2 text-xs font-medium text-accent-400 hover:text-accent-300 cursor-pointer"
          >
            {copied ? "Copied!" : "Copy address"}
          </button>
        </div>
      ) : (
        <p className="mt-4 text-sm text-surface-500">
          Generate a shielded Zcash address to receive deposits.
        </p>
      )}

      <div className="mt-4">
        <Button
          variant="secondary"
          size="sm"
          onClick={generateAddress}
          disabled={loading}
        >
          {loading && <Spinner size="sm" />}
          <span className={loading ? "ml-2" : ""}>
            {address ? "Get New Address" : "Get Deposit Address"}
          </span>
        </Button>
      </div>
    </Card>
  );
}
