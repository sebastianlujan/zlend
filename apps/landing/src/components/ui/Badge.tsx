interface BadgeProps {
  children: React.ReactNode;
}

export function Badge({ children }: BadgeProps) {
  return (
    <span className="inline-block px-3 py-1 text-xs font-medium tracking-wider uppercase rounded-full bg-primary-900/50 text-primary-300 border border-primary-700/30">
      {children}
    </span>
  );
}
