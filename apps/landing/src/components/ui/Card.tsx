interface CardProps {
  children: React.ReactNode;
  className?: string;
}

export function Card({ children, className = "" }: CardProps) {
  return (
    <div
      className={`relative rounded-xl border border-surface-700/50 bg-surface-900/50 backdrop-blur-sm p-6 transition-colors duration-200 hover:border-primary-700/40 ${className}`}
    >
      {children}
    </div>
  );
}
