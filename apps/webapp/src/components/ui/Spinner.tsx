interface SpinnerProps {
  size?: "sm" | "md";
}

const sizes = {
  sm: "h-4 w-4",
  md: "h-5 w-5",
};

export function Spinner({ size = "sm" }: SpinnerProps) {
  return (
    <svg
      className={`animate-spin ${sizes[size]} text-current`}
      viewBox="0 0 24 24"
      fill="none"
    >
      <circle
        className="opacity-25"
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        strokeWidth="4"
      />
      <path
        className="opacity-75"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
      />
    </svg>
  );
}
