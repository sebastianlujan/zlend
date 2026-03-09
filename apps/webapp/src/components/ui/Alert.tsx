const ALERT_VARIANTS = {
  danger: "border-primary-500/50 bg-primary-500/10 text-primary-300",
  warning: "border-yellow-500/50 bg-yellow-500/10 text-yellow-300",
  info: "border-accent-500/50 bg-accent-500/10 text-accent-300",
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
