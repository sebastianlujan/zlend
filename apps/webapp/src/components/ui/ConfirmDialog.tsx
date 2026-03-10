import { Modal } from "./Modal.tsx";
import { InfoRow } from "./InfoRow.tsx";
import { Button } from "./Button.tsx";
import { Spinner } from "./Spinner.tsx";
import { Alert } from "./Alert.tsx";

interface ConfirmItem {
  label: string;
  value: React.ReactNode;
}

interface ConfirmDialogProps {
  open: boolean;
  onClose: () => void;
  onConfirm: () => void;
  title: string;
  items: ConfirmItem[];
  warning?: string;
  confirmLabel?: string;
  loading?: boolean;
}

export function ConfirmDialog({
  open,
  onClose,
  onConfirm,
  title,
  items,
  warning,
  confirmLabel = "Confirm",
  loading = false,
}: ConfirmDialogProps) {
  return (
    <Modal open={open} onClose={onClose} title={title}>
      <div className="space-y-1">
        {items.map((item) => (
          <InfoRow key={item.label} label={item.label} value={item.value} />
        ))}
      </div>

      {warning && (
        <div className="mt-3">
          <Alert variant="warning">{warning}</Alert>
        </div>
      )}

      <div className="mt-5 flex items-center gap-3">
        <Button
          variant="ghost"
          size="sm"
          className="flex-1"
          onClick={onClose}
          disabled={loading}
        >
          Cancel
        </Button>
        <Button
          size="sm"
          className="flex-1"
          onClick={onConfirm}
          disabled={loading}
        >
          {loading && <Spinner size="sm" />}
          <span className={loading ? "ml-2" : ""}>{confirmLabel}</span>
        </Button>
      </div>
    </Modal>
  );
}
