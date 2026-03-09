const BADGE_VARIANTS = {
  default: "bg-surface-700 text-surface-400",
  success: "bg-green-900/50 text-green-400",
  warning: "bg-yellow-900/50 text-yellow-400",
  danger: "bg-primary-900/50 text-primary-400",
  info: "bg-accent-600/20 text-accent-400",
} as const;

type BadgeVariant = keyof typeof BADGE_VARIANTS;

interface BadgeProps {
  children: React.ReactNode;
  variant?: BadgeVariant;
}

export function Badge({ children, variant = "default" }: BadgeProps) {
  return (
    <span
      className={`inline-flex items-center rounded px-1.5 py-0.5 text-[10px] font-medium uppercase ${BADGE_VARIANTS[variant]}`}
    >
      {children}
    </span>
  );
}
