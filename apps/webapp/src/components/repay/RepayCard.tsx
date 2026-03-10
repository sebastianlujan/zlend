import { useState, useEffect } from "react";
import {
  useAccount,
  useReadContracts,
  useWriteContract,
  useWaitForTransactionReceipt,
} from "wagmi";
import { formatUnits, parseUnits } from "viem";
import { Card } from "../ui/Card.tsx";
import { Button } from "../ui/Button.tsx";
import { Spinner } from "../ui/Spinner.tsx";
import { ConfirmDialog } from "../ui/ConfirmDialog.tsx";
import { TxLink } from "../ui/TxLink.tsx";
import type { Vault } from "../../types/vault.ts";
import { ogBankAbi } from "../../config/ogbank-abi.ts";
import { addresses, erc20Abi, erc20ApproveAbi } from "../../config/contracts.ts";
import { useToast } from "../../hooks/useToast.ts";

interface RepayCardProps {
  vault: Vault;
  onRepaid: () => void;
}

export function RepayCard({ vault, onRepaid }: RepayCardProps) {
  const { address, chain } = useAccount();
  const [showConfirm, setShowConfirm] = useState(false);
  const { addToast } = useToast();
  const ogBankAddress = addresses.ogBank;

  const explorerUrl = chain?.blockExplorers?.default?.url;

  const { data: tokenData } = useReadContracts({
    contracts: [
      {
        address: addresses.borrowToken,
        abi: erc20Abi,
        functionName: "balanceOf",
        args: [address!],
      },
      {
        address: addresses.borrowToken,
        abi: erc20Abi,
        functionName: "decimals",
      },
      {
        address: addresses.borrowToken,
        abi: erc20ApproveAbi,
        functionName: "allowance",
        args: [address!, ogBankAddress!],
      },
    ],
    query: { enabled: !!address && !!ogBankAddress },
  });

  const userBalance = tokenData?.[0]?.result as bigint | undefined;
  const decimals = (tokenData?.[1]?.result as number | undefined) ?? 6;
  const allowance = tokenData?.[2]?.result as bigint | undefined;

  const borrowedAmount = vault.borrowedAmount ?? "0";
  const totalDue = parseUnits(borrowedAmount, decimals);

  const userBalanceFormatted =
    userBalance !== undefined
      ? parseFloat(formatUnits(userBalance, decimals)).toFixed(2)
      : "—";

  const hasSufficientBalance =
    userBalance !== undefined && userBalance >= totalDue;
  const needsApproval =
    allowance !== undefined && ogBankAddress && allowance < totalDue;

  const {
    writeContract: writeApprove,
    data: approveTxHash,
    isPending: isApprovePending,
  } = useWriteContract();

  const { isLoading: isApproveConfirming, isSuccess: isApproveSuccess } =
    useWaitForTransactionReceipt({ hash: approveTxHash });

  const {
    writeContract: writeRepay,
    data: repayTxHash,
    isPending: isRepayPending,
  } = useWriteContract();

  const { isLoading: isRepayConfirming, isSuccess: isRepaySuccess } =
    useWaitForTransactionReceipt({ hash: repayTxHash });

  const handleRepay = () => {
    if (!ogBankAddress || !vault.borrowNullifier) return;
    addToast({ variant: "info", title: "Submitting repayment..." });
    writeRepay(
      {
        address: ogBankAddress,
        abi: ogBankAbi,
        functionName: "repay",
        args: [totalDue, vault.borrowNullifier],
      },
      {
        onSuccess: (hash) => {
          onRepaid();
          addToast({
            variant: "success",
            title: "Loan Repaid!",
            message: `${borrowedAmount} USDT repaid successfully`,
            action: explorerUrl && hash
              ? { label: "View on Explorer", onClick: () => window.open(`${explorerUrl}/tx/${hash}`, "_blank") }
              : undefined,
          });
        },
      },
    );
  };

  useEffect(() => {
    if (isApproveSuccess && !repayTxHash && !isRepayPending) {
      addToast({ variant: "success", title: "USDT Approved!", message: "Submitting repayment..." });
      handleRepay();
    }
  }, [isApproveSuccess]);

  const handleClick = () => {
    setShowConfirm(true);
  };

  const handleConfirm = () => {
    setShowConfirm(false);
    if (!ogBankAddress) return;

    if (needsApproval && !isApproveSuccess) {
      addToast({ variant: "info", title: "Approving USDT..." });
      writeApprove({
        address: addresses.borrowToken,
        abi: erc20ApproveAbi,
        functionName: "approve",
        args: [ogBankAddress, totalDue],
      });
      return;
    }

    handleRepay();
  };

  const isDisabled =
    isApprovePending ||
    isApproveConfirming ||
    isRepayPending ||
    isRepayConfirming ||
    isRepaySuccess ||
    !hasSufficientBalance ||
    !ogBankAddress;

  const buttonLabel = isApprovePending || isApproveConfirming
    ? "Approving..."
    : isRepayPending
      ? "Submitting repayment..."
      : isRepayConfirming
        ? "Confirming..."
        : isRepaySuccess
          ? "Repaid!"
          : !hasSufficientBalance
            ? "Insufficient balance"
            : "Repay Loan";

  const isBusy =
    isApprovePending || isApproveConfirming || isRepayPending || isRepayConfirming;

  return (
    <>
      <Card className="p-4">
        <div className="flex items-center justify-between">
          <p className="text-xs font-medium uppercase tracking-wider text-surface-500">
            Repay Loan
          </p>
        </div>

        <div className="mt-3 grid grid-cols-2 gap-3 rounded-lg bg-surface-800/60 p-3">
          <div>
            <p className="text-[11px] uppercase tracking-wide text-surface-500">Principal</p>
            <p className="mt-0.5 font-mono text-sm font-medium text-white">{borrowedAmount} USDT</p>
          </div>
          <div className="text-right">
            <p className="text-[11px] uppercase tracking-wide text-surface-500">Your Balance</p>
            <p className="mt-0.5 font-mono text-sm font-medium text-white">
              {userBalanceFormatted} USDT
            </p>
          </div>
        </div>

        <Button size="sm" className="mt-3 w-full" onClick={handleClick} disabled={isDisabled}>
          {isBusy && <Spinner size="sm" />}
          <span className={isBusy ? "ml-2" : ""}>{buttonLabel}</span>
        </Button>

        {repayTxHash && (
          <div className="mt-2 text-center">
            <TxLink hash={repayTxHash} />
          </div>
        )}
      </Card>

      <ConfirmDialog
        open={showConfirm}
        onClose={() => setShowConfirm(false)}
        onConfirm={handleConfirm}
        title="Confirm Repayment"
        items={[
          { label: "Repay Amount", value: `${borrowedAmount} USDT` },
          { label: "Your Balance", value: `${userBalanceFormatted} USDT` },
        ]}
        warning="Full repayment required in a single transaction. Partial repayment will lock your vault."
        confirmLabel="Repay Loan"
      />
    </>
  );
}
