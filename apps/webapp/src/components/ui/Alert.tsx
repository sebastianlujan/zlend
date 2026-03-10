const ALERT_VARIANTS = {
  danger: "border-primary-700/30 bg-primary-900/20 text-primary-300",
  warning: "border-yellow-700/30 bg-yellow-900/20 text-yellow-300",
  info: "border-accent-500/20 bg-accent-500/10 text-accent-400",
} as const;

type AlertVariant = keyof typeof ALERT_VARIANTS;

interface AlertProps {
  children: React.ReactNode;
  variant?: AlertVariant;
}

export function Alert({ children, variant = "info" }: AlertProps) {
  return (
    <div
      className={`rounded-lg border p-3 text-sm ${ALERT_VARIANTS[variant]}`}
    >
      {children}
    </div>
  );
}
