import { useState } from "react";
import { Card } from "../ui/Card.tsx";
import { Badge } from "../ui/Badge.tsx";
import { Button } from "../ui/Button.tsx";
import { AmountInput } from "../ui/AmountInput.tsx";
import { Spinner } from "../ui/Spinner.tsx";

interface DepositFormProps {
  onDeposit: (zecAmount: string, shieldedAddress: string) => void;
}

export function DepositForm({ onDeposit }: DepositFormProps) {
  const [amount, setAmount] = useState("");
  const [loading, setLoading] = useState(false);

  const handleDeposit = async () => {
    if (!amount || parseFloat(amount) <= 0) return;
    setLoading(true);
    await new Promise((r) => setTimeout(r, 1500));
    onDeposit(amount, "");
    setAmount("");
    setLoading(false);
  };

  return (
    <Card className="border-dashed border-surface-600">
      <div className="flex items-center gap-2">
        <p className="text-xs font-medium uppercase tracking-wider text-surface-500">
          Deposit ZEC
        </p>
        <Badge>Simulated</Badge>
      </div>

      <div className="mt-4">
        <AmountInput
          value={amount}
          onChange={setAmount}
          symbol="ZEC"
          disabled={loading}
          placeholder="0.00"
        />
      </div>

      <div className="mt-4">
        <Button
          size="sm"
          onClick={handleDeposit}
          disabled={loading || !amount || parseFloat(amount) <= 0}
        >
          {loading && <Spinner size="sm" />}
          <span className={loading ? "ml-2" : ""}>
            {loading ? "Depositing..." : "Deposit ZEC"}
          </span>
        </Button>
      </div>
    </Card>
  );
}
