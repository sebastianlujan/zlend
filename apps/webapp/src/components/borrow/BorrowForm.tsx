import { useState } from "react";
import { useWriteContract, useWaitForTransactionReceipt } from "wagmi";
import { parseUnits, type Hex } from "viem";
import { Card } from "../ui/Card.tsx";
import { Button } from "../ui/Button.tsx";
import { AmountInput } from "../ui/AmountInput.tsx";
import { InfoRow } from "../ui/InfoRow.tsx";
import { Spinner } from "../ui/Spinner.tsx";
import { VAULT_STATUS, type Vault } from "../../types/vault.ts";
import { ogBankAbi } from "../../config/ogbank-abi.ts";
import { addresses } from "../../config/contracts.ts";

interface BorrowFormProps {
  vault: Vault;
  onBorrowed: (borrowNullifier: Hex, amount: string) => void;
}

export function BorrowForm({ vault, onBorrowed }: BorrowFormProps) {
  const [amount, setAmount] = useState("");
  const isProofReady = vault.status === VAULT_STATUS.PROOF_READY;
  const ogBankAddress = addresses.ogBank;

  const { writeContract, data: txHash, isPending } = useWriteContract();

  const { isLoading: isConfirming, isSuccess } = useWaitForTransactionReceipt({
    hash: txHash,
  });

  const handleBorrow = () => {
    if (!vault.proofData || !ogBankAddress || !amount) return;

    writeContract(
      {
        address: ogBankAddress,
        abi: ogBankAbi,
        functionName: "borrow",
        args: [
          vault.proofData.proof,
          vault.proofData.publicInputs,
          parseUnits(amount, 6),
        ],
      },
      {
        onSuccess: () => {
          onBorrowed(vault.proofData!.publicInputs[0] as Hex, amount);
          setAmount("");
        },
      },
    );
  };

  const isDisabled =
    !isProofReady ||
    isPending ||
    isConfirming ||
    !amount ||
    parseFloat(amount) <= 0 ||
    !ogBankAddress;

  return (
    <Card>
      <p className="text-xs font-medium uppercase tracking-wider text-surface-400">
        Borrow USDT
      </p>

      <div className="mt-4">
        <AmountInput
          value={amount}
          onChange={setAmount}
          symbol="USDT"
          disabled={!isProofReady || isPending || isConfirming}
          placeholder="0.00"
        />
      </div>

      <div className="mt-4 space-y-1">
        <InfoRow label="Rate" value="Aave Variable" />
        <InfoRow label="Collateral" value={`${vault.zecAmount} ZEC`} />
        {!ogBankAddress && (
          <p className="text-xs text-primary-400">OGBank contract not configured</p>
        )}
      </div>

      <div className="mt-4">
        <Button size="sm" onClick={handleBorrow} disabled={isDisabled}>
          {(isPending || isConfirming) && <Spinner size="sm" />}
          <span className={isPending || isConfirming ? "ml-2" : ""}>
            {isPending
              ? "Submitting..."
              : isConfirming
                ? "Confirming..."
                : isSuccess
                  ? "Borrowed!"
                  : !isProofReady
                    ? "Generate proof first"
                    : "Borrow USDT"}
          </span>
        </Button>
      </div>
    </Card>
  );
}
