import { useState } from "react";
import {
  useAccount,
  useWriteContract,
  useWaitForTransactionReceipt,
} from "wagmi";
import { parseUnits } from "viem";
import { Card } from "../ui/Card.tsx";
import { Badge } from "../ui/Badge.tsx";
import { Button } from "../ui/Button.tsx";
import { AmountInput } from "../ui/AmountInput.tsx";
import { Spinner } from "../ui/Spinner.tsx";
import { Alert } from "../ui/Alert.tsx";
import { AddressDisplay } from "../ui/AddressDisplay.tsx";
import { ConfirmDialog } from "../ui/ConfirmDialog.tsx";
import type { Vault } from "../../types/vault.ts";
import { ogBankAbi } from "../../config/ogbank-abi.ts";
import { addresses } from "../../config/contracts.ts";
import { useProofGeneration } from "../../hooks/useProofGeneration.ts";
import { useToast } from "../../hooks/useToast.ts";

interface WithdrawCardProps {
  vault: Vault;
  onWithdrawn: () => void;
}

export function WithdrawCard({ vault, onWithdrawn }: WithdrawCardProps) {
  const { address, chain } = useAccount();
  const [amount, setAmount] = useState(vault.zecAmount);
  const [showConfirm, setShowConfirm] = useState(false);
  const { addToast } = useToast();
  const ogBankAddress = addresses.ogBank;

  const explorerUrl = chain?.blockExplorers?.default?.url;

  const {
    generateAuth,
    isGenerating,
    error: proofError,
  } = useProofGeneration();
  const { writeContract, data: txHash, isPending } = useWriteContract();
  const { isLoading: isConfirming } = useWaitForTransactionReceipt({
    hash: txHash,
  });

  const hasSecrets = !!vault.userSecret && !!vault.nonce;

  const handleWithdraw = async () => {
    setShowConfirm(false);

    if (!ogBankAddress || !amount || !address) return;
    if (!vault.borrowNullifier || !vault.userSecret || !vault.nonce) return;

    addToast({ variant: "info", title: "Generating auth proof..." });

    const result = await generateAuth({
      userSecret: vault.userSecret,
      nonce: vault.nonce,
      borrowNullifier: vault.borrowNullifier,
      recipient: address,
    });

    if (!result) return;

    addToast({ variant: "info", title: "Submitting withdrawal..." });

    writeContract(
      {
        address: ogBankAddress,
        abi: ogBankAbi,
        functionName: "withdrawProof",
        args: [result.proof, result.publicInputs, parseUnits(amount, 8)],
      },
      {
        onSuccess: (hash) => {
          onWithdrawn();
          addToast({
            variant: "success",
            title: "ZEC Withdrawn!",
            message: `${amount} ZEC returned to your shielded address`,
            action: explorerUrl && hash
              ? { label: "View on Explorer", onClick: () => window.open(`${explorerUrl}/tx/${hash}`, "_blank") }
              : undefined,
          });
        },
      },
    );
  };

  const busy = isGenerating || isPending || isConfirming;

  return (
    <>
      <Card>
        <div className="flex items-center gap-2">
          <p className="text-xs font-medium uppercase tracking-wider text-surface-500">
            Withdraw ZEC
          </p>
          <Badge variant="info">Real Proof</Badge>
        </div>

        <div className="mt-4">
          <AmountInput
            value={amount}
            onChange={setAmount}
            symbol="ZEC"
            maxAmount={vault.zecAmount}
            disabled={busy}
          />
        </div>

        {!hasSecrets && (
          <div className="mt-3">
            <Alert variant="warning">
              This vault was created before automatic secret generation. Please
              create a new vault.
            </Alert>
          </div>
        )}

        {proofError && (
          <div className="mt-3">
            <Alert variant="danger">{proofError}</Alert>
          </div>
        )}

        <div className="mt-4 space-y-1">
          <div className="flex items-center justify-between py-1">
            <span className="text-sm text-surface-400">Return to</span>
            <AddressDisplay
              address={vault.shieldedAddress}
              startChars={12}
              endChars={6}
            />
          </div>
          <p className="text-xs text-surface-500">
            The relayer will return ZEC to your shielded address.
          </p>
        </div>

        <div className="mt-4">
          <Button
            size="sm"
            onClick={() => setShowConfirm(true)}
            disabled={
              busy ||
              !amount ||
              parseFloat(amount) <= 0 ||
              !ogBankAddress ||
              !hasSecrets
            }
          >
            {busy && <Spinner size="sm" />}
            <span className={busy ? "ml-2" : ""}>
              {isGenerating
                ? "Generating proof…"
                : isPending
                  ? "Submitting…"
                  : isConfirming
                    ? "Confirming…"
                    : "Withdraw ZEC"}
            </span>
          </Button>
        </div>
      </Card>

      <ConfirmDialog
        open={showConfirm}
        onClose={() => setShowConfirm(false)}
        onConfirm={handleWithdraw}
        title="Confirm Withdrawal"
        items={[
          { label: "Amount", value: `${amount} ZEC` },
          {
            label: "Return to",
            value: `${vault.shieldedAddress.slice(0, 12)}...${vault.shieldedAddress.slice(-6)}`,
          },
        ]}
        confirmLabel="Withdraw ZEC"
      />
    </>
  );
}
