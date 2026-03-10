import { useEffect, useRef } from "react";

interface ModalProps {
  open: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
}

export function Modal({ open, onClose, title, children }: ModalProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;

    if (open && !dialog.open) {
      dialog.showModal();
    } else if (!open && dialog.open) {
      dialog.close();
    }
  }, [open]);

  return (
    <dialog
      ref={dialogRef}
      onClose={onClose}
      className="fixed inset-0 m-auto w-full max-w-lg rounded-xl border border-surface-700/50 bg-surface-900 p-8 text-surface-100 backdrop:bg-black/60 backdrop:backdrop-blur-sm"
    >
      <h3 className="mb-4 text-lg font-semibold text-white">{title}</h3>
      {children}
    </dialog>
  );
}
