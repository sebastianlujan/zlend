import { useState, useEffect } from "react";
import { useAccount, useWriteContract, useWaitForTransactionReceipt } from "wagmi";
import { parseUnits, type Hex } from "viem";
import { Card } from "../ui/Card.tsx";
import { Button } from "../ui/Button.tsx";
import { Spinner } from "../ui/Spinner.tsx";
import { Alert } from "../ui/Alert.tsx";
import { Skeleton } from "../ui/Skeleton.tsx";
import { ConfirmDialog } from "../ui/ConfirmDialog.tsx";
import { TxLink } from "../ui/TxLink.tsx";
import { VAULT_STATUS, type Vault } from "../../types/vault.ts";
import { useProofGeneration } from "../../hooks/useProofGeneration.ts";
import { useZecPrice } from "../../hooks/useZecPrice.ts";
import { useToast } from "../../hooks/useToast.ts";
import { ogBankAbi } from "../../config/ogbank-abi.ts";
import { addresses } from "../../config/contracts.ts";

interface GenerateUSDTCardProps {
  vault: Vault;
  onProofGenerated: (proofData: { proof: Hex; publicInputs: Hex[] }) => void;
  onBorrowed: (borrowNullifier: Hex, amount: string) => void;
}

export function GenerateUSDTCard({
  vault,
  onProofGenerated,
  onBorrowed,
}: GenerateUSDTCardProps) {
  const { address, chain } = useAccount();
  const { generateBorrow, isGenerating, error: proofError } = useProofGeneration();
  const [proofReady, setProofReady] = useState(vault.status === VAULT_STATUS.PROOF_READY);
  const [showConfirm, setShowConfirm] = useState(false);
  const { addToast } = useToast();

  const { price, loading: priceLoading, error: priceError } = useZecPrice();

  const ogBankAddress = addresses.ogBank;
  const usdtAmount = price
    ? (parseFloat(vault.zecAmount) * price).toFixed(2)
    : null;
  const hasSecrets = !!vault.userSecret && !!vault.nonce;

  const { writeContract, data: txHash, isPending } = useWriteContract();
  const { isLoading: isConfirming, isSuccess } = useWaitForTransactionReceipt({
    hash: txHash,
  });

  const explorerUrl = chain?.blockExplorers?.default?.url;

  const sendBorrowTx = () => {
    if (!vault.proofData || !ogBankAddress || !usdtAmount) return;

    addToast({ variant: "info", title: "Submitting transaction..." });
    writeContract(
      {
        address: ogBankAddress,
        abi: ogBankAbi,
        functionName: "borrow",
        args: [
          vault.proofData.proof,
          vault.proofData.publicInputs,
          parseUnits(usdtAmount, 6),
        ],
      },
      {
        onSuccess: (hash) => {
          onBorrowed(vault.proofData!.publicInputs[2] as Hex, usdtAmount!);
          addToast({
            variant: "success",
            title: "USDT Generated!",
            message: `${usdtAmount} USDT borrowed against ${vault.zecAmount} ZEC`,
            action: explorerUrl && hash
              ? { label: "View on Explorer", onClick: () => window.open(`${explorerUrl}/tx/${hash}`, "_blank") }
              : undefined,
          });
        },
      },
    );
  };

  useEffect(() => {
    if (proofReady && vault.proofData && !txHash && !isPending) {
      sendBorrowTx();
    }
  }, [proofReady, vault.proofData]);

  const handleClick = () => {
    setShowConfirm(true);
  };

  const handleConfirm = async () => {
    setShowConfirm(false);

    if (vault.status === VAULT_STATUS.PROOF_READY && vault.proofData) {
      sendBorrowTx();
      return;
    }

    if (!vault.userSecret || !vault.nonce || !address) return;

    addToast({ variant: "info", title: "Generating ZK proof..." });
    const zecSatoshis = Math.floor(parseFloat(vault.zecAmount) * 1e8).toString();

    const result = await generateBorrow({
      userSecret: vault.userSecret,
      value: zecSatoshis,
      nonce: vault.nonce,
      threshold: zecSatoshis,
      recipient: address,
    });

    if (result) {
      setProofReady(true);
      addToast({ variant: "success", title: "Proof generated!", message: "Submitting transaction..." });
      onProofGenerated(result);
    }
  };

  const isDisabled =
    isGenerating || isPending || isConfirming || isSuccess || !hasSecrets || !ogBankAddress || !usdtAmount;

  const buttonLabel = isGenerating
    ? "Generating proof..."
    : isPending
      ? "Submitting..."
      : isConfirming
        ? "Confirming..."
        : isSuccess
          ? "USDT Generated!"
          : "Generate USDT";

  return (
    <>
      <Card className="p-4">
        <div className="flex items-center justify-between">
          <p className="text-xs font-medium uppercase tracking-wider text-surface-500">
            Generate USDT
          </p>
          <span className="rounded bg-surface-800 px-2 py-0.5 font-mono text-[11px] text-surface-400">
            {priceLoading ? (
              <Skeleton className="inline-block h-3 w-20" />
            ) : priceError ? "—" : `1 ZEC ≈ $${price?.toFixed(2)}`}
          </span>
        </div>

        <div className="mt-3 grid grid-cols-2 gap-3 rounded-lg bg-surface-800/60 p-3">
          <div>
            <p className="text-[11px] uppercase tracking-wide text-surface-500">Collateral</p>
            <p className="mt-0.5 font-mono text-sm font-medium text-white">{vault.zecAmount} ZEC</p>
          </div>
          <div className="text-right">
            <p className="text-[11px] uppercase tracking-wide text-surface-500">You receive</p>
            <p className="mt-0.5 font-mono text-sm font-semibold text-emerald-400">
              {priceLoading ? <Skeleton className="ml-auto h-4 w-16" /> : `${usdtAmount ?? "–"} USDT`}
            </p>
          </div>
        </div>

        {!ogBankAddress && (
          <p className="mt-2 text-xs text-primary-400">OGBank contract not configured</p>
        )}

        {!hasSecrets && (
          <div className="mt-2">
            <Alert variant="warning">
              Vault created before automatic secret generation. Create a new vault.
            </Alert>
          </div>
        )}

        {proofError && (
          <div className="mt-2">
            <Alert variant="danger">{proofError}</Alert>
          </div>
        )}

        <Button size="sm" className="mt-3 w-full" onClick={handleClick} disabled={isDisabled}>
          {(isGenerating || isPending || isConfirming) && <Spinner size="sm" />}
          <span className={isGenerating || isPending || isConfirming ? "ml-2" : ""}>
            {buttonLabel}
          </span>
        </Button>

        {txHash && (
          <div className="mt-2 text-center">
            <TxLink hash={txHash} />
          </div>
        )}
      </Card>

      <ConfirmDialog
        open={showConfirm}
        onClose={() => setShowConfirm(false)}
        onConfirm={handleConfirm}
        title="Confirm Borrow"
        items={[
          { label: "Collateral", value: `${vault.zecAmount} ZEC` },
          { label: "You receive", value: `${usdtAmount ?? "–"} USDT` },
          { label: "ZEC Price", value: price ? `$${price.toFixed(2)}` : "–" },
        ]}
        confirmLabel="Generate USDT"
      />
    </>
  );
}
