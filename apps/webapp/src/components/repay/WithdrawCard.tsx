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
import { InfoRow } from "../ui/InfoRow.tsx";
import { Spinner } from "../ui/Spinner.tsx";
import { Alert } from "../ui/Alert.tsx";
import type { Vault } from "../../types/vault.ts";
import { ogBankAbi } from "../../config/ogbank-abi.ts";
import { addresses } from "../../config/contracts.ts";
import { useProofGeneration } from "../../hooks/useProofGeneration.ts";

interface WithdrawCardProps {
  vault: Vault;
  onWithdrawn: () => void;
}

export function WithdrawCard({ vault, onWithdrawn }: WithdrawCardProps) {
  const { address } = useAccount();
  const [amount, setAmount] = useState(vault.zecAmount);
  const ogBankAddress = addresses.ogBank;

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
    if (!ogBankAddress || !amount || !address) return;
    if (!vault.borrowNullifier || !vault.userSecret || !vault.nonce) return;

    const result = await generateAuth({
      userSecret: vault.userSecret,
      nonce: vault.nonce,
      borrowNullifier: vault.borrowNullifier,
      recipient: address,
    });

    if (!result) return;

    writeContract(
      {
        address: ogBankAddress,
        abi: ogBankAbi,
        functionName: "withdrawProof",
        args: [result.proof, result.publicInputs, parseUnits(amount, 8)],
      },
      {
        onSuccess: () => {
          onWithdrawn();
        },
      },
    );
  };

  const busy = isGenerating || isPending || isConfirming;

  return (
    <Card>
      <div className="flex items-center gap-2">
        <p className="text-xs font-medium uppercase tracking-wider text-surface-400">
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
        <InfoRow
          label="Return to"
          value={`${vault.shieldedAddress.slice(0, 12)}...${vault.shieldedAddress.slice(-6)}`}
        />
        <p className="text-xs text-surface-500">
          The relayer will return ZEC to your shielded address.
        </p>
      </div>

      <div className="mt-4">
        <Button
          size="sm"
          onClick={handleWithdraw}
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
  );
}
