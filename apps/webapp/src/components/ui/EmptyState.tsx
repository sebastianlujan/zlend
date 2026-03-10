import type { LucideIcon } from "lucide-react";

interface EmptyStateProps {
  icon: LucideIcon;
  heading: string;
  description: string;
  children?: React.ReactNode;
}

export function EmptyState({ icon: Icon, heading, description, children }: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center py-12 text-center">
      <div className="flex h-14 w-14 items-center justify-center rounded-full bg-surface-800 text-surface-500">
        <Icon size={28} />
      </div>
      <h3 className="mt-4 text-sm font-medium text-surface-300">{heading}</h3>
      <p className="mt-1.5 max-w-xs text-xs text-surface-500">{description}</p>
      {children && <div className="mt-4">{children}</div>}
    </div>
  );
}
