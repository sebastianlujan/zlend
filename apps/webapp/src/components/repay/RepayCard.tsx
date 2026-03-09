import { useState } from "react";
import {
  useAccount,
  useReadContracts,
  useWriteContract,
  useWaitForTransactionReceipt,
} from "wagmi";
import { formatUnits, parseUnits } from "viem";
import { Card } from "../ui/Card.tsx";
import { Button } from "../ui/Button.tsx";
import { Alert } from "../ui/Alert.tsx";
import { InfoRow } from "../ui/InfoRow.tsx";
import { Spinner } from "../ui/Spinner.tsx";
import { Modal } from "../ui/Modal.tsx";
import type { Vault } from "../../types/vault.ts";
import { ogBankAbi } from "../../config/ogbank-abi.ts";
import { addresses, erc20Abi, erc20ApproveAbi } from "../../config/contracts.ts";

interface RepayCardProps {
  vault: Vault;
  onRepaid: () => void;
}

export function RepayCard({ vault, onRepaid }: RepayCardProps) {
  const { address } = useAccount();
  const [showModal, setShowModal] = useState(false);
  const ogBankAddress = addresses.ogBank;

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

  // Total due = borrowed amount (in production would query Aave debt)
  const borrowedAmount = vault.borrowedAmount ?? "0";
  const totalDue = parseUnits(borrowedAmount, decimals);
  const totalDueFormatted = borrowedAmount;

  const userBalanceFormatted =
    userBalance !== undefined
      ? parseFloat(formatUnits(userBalance, decimals)).toFixed(2)
      : "—";

  const hasSufficientBalance =
    userBalance !== undefined && userBalance >= totalDue;
  const needsApproval =
    allowance !== undefined && ogBankAddress && allowance < totalDue;

  // Approve tx
  const {
    writeContract: writeApprove,
    data: approveTxHash,
    isPending: isApprovePending,
  } = useWriteContract();

  const { isLoading: isApproveConfirming, isSuccess: isApproveSuccess } =
    useWaitForTransactionReceipt({ hash: approveTxHash });

  // Repay tx
  const {
    writeContract: writeRepay,
    data: repayTxHash,
    isPending: isRepayPending,
  } = useWriteContract();

  const { isLoading: isRepayConfirming } = useWaitForTransactionReceipt({
    hash: repayTxHash,
  });

  const handleApprove = () => {
    if (!ogBankAddress) return;
    writeApprove({
      address: addresses.borrowToken,
      abi: erc20ApproveAbi,
      functionName: "approve",
      args: [ogBankAddress, totalDue],
    });
  };

  const handleRepay = () => {
    if (!ogBankAddress || !vault.borrowNullifier) return;
    writeRepay(
      {
        address: ogBankAddress,
        abi: ogBankAbi,
        functionName: "repay",
        args: [totalDue, vault.borrowNullifier],
      },
      {
        onSuccess: () => {
          setShowModal(false);
          onRepaid();
        },
      },
    );
  };

  const showApproveButton = needsApproval && !isApproveSuccess;

  return (
    <>
      <Card>
        <p className="text-xs font-medium uppercase tracking-wider text-surface-400">
          Repay Loan
        </p>

        <div className="mt-4">
          <Alert variant="danger">
            <strong>Important:</strong> Repayment must be made in FULL in a
            single transaction. Partial repayment will permanently lock your
            vault. This action cannot be undone.
          </Alert>
        </div>

        <div className="mt-4 space-y-1">
          <InfoRow label="Principal" value={`${borrowedAmount} USDT`} />
          <InfoRow label="Accrued Interest" value="~0.00 USDT" />
          <div className="border-t border-surface-700/50 pt-1">
            <InfoRow
              label="Total Due"
              value={`${totalDueFormatted} USDT`}
              bold
            />
          </div>
          <InfoRow
            label="Your Balance"
            value={
              <span className="flex items-center gap-2">
                {userBalanceFormatted} USDT
                {userBalance !== undefined && (
                  <span
                    className={`text-xs ${hasSufficientBalance ? "text-green-400" : "text-primary-400"}`}
                  >
                    {hasSufficientBalance ? "Sufficient" : "Insufficient"}
                  </span>
                )}
              </span>
            }
          />
        </div>

        <div className="mt-4 flex items-center gap-3">
          {showApproveButton && (
            <Button
              variant="secondary"
              size="sm"
              onClick={handleApprove}
              disabled={isApprovePending || isApproveConfirming}
            >
              {(isApprovePending || isApproveConfirming) && (
                <Spinner size="sm" />
              )}
              <span
                className={
                  isApprovePending || isApproveConfirming ? "ml-2" : ""
                }
              >
                {isApprovePending
                  ? "Approving..."
                  : isApproveConfirming
                    ? "Confirming..."
                    : "Approve USDT"}
              </span>
            </Button>
          )}

          <Button
            size="sm"
            onClick={() => setShowModal(true)}
            disabled={
              !hasSufficientBalance ||
              (needsApproval && !isApproveSuccess) ||
              isRepayPending ||
              isRepayConfirming ||
              !ogBankAddress
            }
          >
            {(isRepayPending || isRepayConfirming) && <Spinner size="sm" />}
            <span
              className={isRepayPending || isRepayConfirming ? "ml-2" : ""}
            >
              {isRepayPending
                ? "Submitting..."
                : isRepayConfirming
                  ? "Confirming..."
                  : `Repay Full Amount (${totalDueFormatted} USDT)`}
            </span>
          </Button>
        </div>
      </Card>

      <Modal
        open={showModal}
        onClose={() => setShowModal(false)}
        title="Confirm Full Repayment"
      >
        <p className="text-sm text-surface-300">
          You are about to repay:
        </p>
        <p className="my-4 text-center font-mono text-2xl font-bold text-white">
          {totalDueFormatted} USDT
        </p>
        <p className="text-sm text-surface-400">
          This is the FULL amount including accrued interest. After repayment,
          you can withdraw your ZEC.
        </p>
        <div className="mt-3">
          <Alert variant="danger">This action cannot be undone.</Alert>
        </div>
        <div className="mt-6 flex justify-end gap-3">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setShowModal(false)}
          >
            Cancel
          </Button>
          <Button size="sm" onClick={handleRepay} disabled={isRepayPending}>
            {isRepayPending && <Spinner size="sm" />}
            <span className={isRepayPending ? "ml-2" : ""}>
              Confirm Repayment
            </span>
          </Button>
        </div>
      </Modal>
    </>
  );
}
