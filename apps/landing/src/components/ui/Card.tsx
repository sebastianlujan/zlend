interface CardProps {
  children: React.ReactNode;
  className?: string;
}

export function Card({ children, className = "" }: CardProps) {
  return (
    <div
      className={`rounded-xl border border-surface-700/50 bg-surface-900/50 backdrop-blur-sm p-6 ${className}`}
    >
      {children}
    </div>
  );
}
