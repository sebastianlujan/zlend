interface SkeletonProps {
  className?: string;
  variant?: "text" | "card" | "circle";
}

const variants = {
  text: "h-4 rounded",
  card: "h-32 rounded-xl",
  circle: "rounded-full",
};

export function Skeleton({ className = "", variant = "text" }: SkeletonProps) {
  return (
    <div
      className={`animate-pulse bg-surface-700/50 ${variants[variant]} ${className}`}
    />
  );
}
