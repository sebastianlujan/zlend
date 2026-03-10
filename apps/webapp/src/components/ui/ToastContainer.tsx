import { createElement } from "react";
import { X, CheckCircle, AlertTriangle, AlertCircle, Info } from "lucide-react";
import { ToastContext, useToast, useToastState, type Toast } from "../../hooks/useToast.ts";

const TOAST_VARIANTS = {
  success: {
    bg: "border-green-700/30 bg-green-900/30",
    text: "text-green-300",
    icon: CheckCircle,
    iconColor: "text-green-400",
  },
  danger: {
    bg: "border-primary-700/30 bg-primary-900/30",
    text: "text-primary-300",
    icon: AlertCircle,
    iconColor: "text-primary-400",
  },
  warning: {
    bg: "border-yellow-700/30 bg-yellow-900/30",
    text: "text-yellow-300",
    icon: AlertTriangle,
    iconColor: "text-yellow-400",
  },
  info: {
    bg: "border-accent-500/20 bg-accent-500/15",
    text: "text-accent-400",
    icon: Info,
    iconColor: "text-accent-400",
  },
} as const;

function ToastItem({ toast, onRemove }: { toast: Toast; onRemove: () => void }) {
  const v = TOAST_VARIANTS[toast.variant];
  const Icon = v.icon;

  return (
    <div
      className={`flex items-start gap-3 rounded-lg border p-3 backdrop-blur-md shadow-lg animate-in slide-in-from-right ${v.bg}`}
      style={{ animation: "slideIn 0.2s ease-out" }}
    >
      <Icon size={18} className={`mt-0.5 shrink-0 ${v.iconColor}`} />
      <div className="min-w-0 flex-1">
        <p className={`text-sm font-medium ${v.text}`}>{toast.title}</p>
        {toast.message && (
          <p className="mt-0.5 text-xs text-surface-400">{toast.message}</p>
        )}
        {toast.action && (
          <button
            onClick={toast.action.onClick}
            className="mt-1 text-xs font-medium text-accent-400 hover:text-accent-300 cursor-pointer transition-colors"
          >
            {toast.action.label}
          </button>
        )}
      </div>
      <button
        onClick={onRemove}
        className="shrink-0 cursor-pointer rounded p-0.5 text-surface-500 hover:text-surface-300 transition-colors"
      >
        <X size={14} />
      </button>
    </div>
  );
}

function ToastList() {
  const { toasts, removeToast } = useToast();

  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-4 right-4 z-[100] flex flex-col gap-2 w-80">
      {toasts.map((toast) => (
        <ToastItem key={toast.id} toast={toast} onRemove={() => removeToast(toast.id)} />
      ))}
    </div>
  );
}

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const state = useToastState();

  return createElement(
    ToastContext.Provider,
    { value: state },
    children,
    createElement(ToastList, null),
  );
}
