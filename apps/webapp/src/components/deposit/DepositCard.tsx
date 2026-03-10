import { useEffect, useState, useCallback } from "react";
import { Card } from "../ui/Card.tsx";
import { Button } from "../ui/Button.tsx";
import { AmountInput } from "../ui/AmountInput.tsx";
import { Spinner } from "../ui/Spinner.tsx";
import { Skeleton } from "../ui/Skeleton.tsx";
import { AddressDisplay } from "../ui/AddressDisplay.tsx";
import { RefreshCw } from "lucide-react";
import { useToast } from "../../hooks/useToast.ts";

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

interface DepositCardProps {
  onDeposit: (zecAmount: string, shieldedAddress: string) => string;
}

export function DepositCard({ onDeposit }: DepositCardProps) {
  const [address, setAddress] = useState("");
  const [addrLoading, setAddrLoading] = useState(false);
  const [error, setError] = useState("");
  const [vaultData, setVaultData] = useState<VaultResponse | null>(null);

  const [amount, setAmount] = useState("");
  const [depositLoading, setDepositLoading] = useState(false);
  const { addToast } = useToast();

  const fetchAddress = useCallback(async () => {
    setAddrLoading(true);
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
      setAddrLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchAddress();
  }, [fetchAddress]);

  const handleDeposit = async () => {
    if (!amount || parseFloat(amount) <= 0) return;
    setDepositLoading(true);
    await new Promise((r) => setTimeout(r, 1500));
    const depositedAmount = amount;
    const vaultId = onDeposit(amount, "");
    setAmount("");
    setDepositLoading(false);
    addToast({
      variant: "success",
      title: "ZEC Deposited!",
      message: `${depositedAmount} ZEC deposited — Vault #${vaultId.slice(0, 6)}`,
    });
  };

  return (
    <Card>
      <p className="text-xs font-medium uppercase tracking-wider text-surface-500">
        Deposit ZEC
      </p>

      <div className="mt-4">
        <AmountInput
          value={amount}
          onChange={setAmount}
          symbol="ZEC"
          disabled={depositLoading}
          placeholder="0.00"
        />
      </div>

      <div className="mt-3 flex items-center gap-2 rounded-lg bg-surface-800/50 px-3 py-2">
        <span className="shrink-0 text-xs text-surface-500">To:</span>
        {addrLoading ? (
          <div className="flex-1">
            <Skeleton className="h-4 w-48" />
          </div>
        ) : error ? (
          <p className="flex-1 text-xs text-red-400">{error}</p>
        ) : address ? (
          <AddressDisplay address={address} className="flex-1" />
        ) : (
          <span className="flex-1 text-xs text-surface-500">No address</span>
        )}

        <button
          onClick={fetchAddress}
          disabled={addrLoading}
          className="shrink-0 cursor-pointer rounded p-1 text-surface-400 hover:text-surface-200 transition-colors disabled:opacity-50"
          title="New address"
        >
          <RefreshCw size={14} className={addrLoading ? "animate-spin" : ""} />
        </button>
      </div>

      {vaultData && !addrLoading && (
        <p className="mt-1.5 text-[10px] text-surface-500 font-mono">
          Vault {vaultData.vault_id.slice(0, 8)}...
        </p>
      )}

      <div className="mt-4">
        <Button
          className="w-full"
          onClick={handleDeposit}
          disabled={depositLoading || !amount || parseFloat(amount) <= 0}
        >
          {depositLoading && <Spinner size="sm" />}
          <span className={depositLoading ? "ml-2" : ""}>
            {depositLoading ? "Depositing..." : "Deposit ZEC"}
          </span>
        </Button>
      </div>
    </Card>
  );
}
