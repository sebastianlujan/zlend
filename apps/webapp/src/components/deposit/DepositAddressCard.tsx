import { useEffect, useState, useCallback } from "react";
import { Card } from "../ui/Card.tsx";
import { Spinner } from "../ui/Spinner.tsx";
import { RefreshCw } from "lucide-react";

interface VaultResponse {
  vault_id: string;
  vault_address: string;
  fvk: string;
  ivk: string;
  user_secret: string;
  nonce: string;
  mnemonic: string;
  status: string;
}

function truncateAddress(addr: string): string {
  if (addr.length <= 16) return addr;
  return `${addr.slice(0, 8)}...${addr.slice(-8)}`;
}

export function DepositAddressCard() {
  const [address, setAddress] = useState("");
  const [loading, setLoading] = useState(false);
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState("");
  const [vaultData, setVaultData] = useState<VaultResponse | null>(null);

  const fetchAddress = useCallback(async () => {
    setLoading(true);
    setError("");
    try {
      const res = await fetch("http://localhost:3000/vault/create", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: "{}",
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data: VaultResponse = await res.json();
      setAddress(data.vault_address);
      setVaultData(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to fetch address");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchAddress();
  }, [fetchAddress]);

  const copyAddress = async () => {
    if (!address) return;
    await navigator.clipboard.writeText(address);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <Card className="border-dashed border-surface-600">
      <p className="text-xs font-medium uppercase tracking-wider text-surface-500">
        Deposit Address
      </p>

      <div className="mt-4">
        <div className="flex items-center gap-2 rounded-lg bg-surface-800/50 px-3 py-2.5">
          {loading ? (
            <div className="flex flex-1 items-center justify-center py-0.5">
              <Spinner size="sm" />
            </div>
          ) : error ? (
            <p className="flex-1 text-xs text-red-500">{error}</p>
          ) : address ? (
            <button
              onClick={copyAddress}
              className="flex-1 cursor-pointer text-left font-mono text-sm text-surface-300 hover:text-white transition-colors"
              title="Click to copy full address"
            >
              {copied ? "Copied!" : truncateAddress(address)}
            </button>
          ) : (
            <p className="flex-1 text-sm text-surface-500">No address</p>
          )}

          <button
            onClick={fetchAddress}
            disabled={loading}
            className="shrink-0 cursor-pointer rounded-md p-1 text-surface-400 hover:bg-surface-700 hover:text-surface-300 transition-colors disabled:opacity-50"
            title="Generate new address"
          >
            <RefreshCw size={16} className={loading ? "animate-spin" : ""} />
          </button>
        </div>
      </div>

      {vaultData && !loading && (
        <p className="mt-2 text-xs text-surface-400">
          Vault: {vaultData.vault_id.slice(0, 8)}...
        </p>
      )}
    </Card>
  );
}
