import { useRef, type ButtonHTMLAttributes } from "react";
import { useMagneticHover } from "../../hooks/useMagneticHover";
import { useReducedMotion } from "../../hooks/useReducedMotion";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "ghost";
  size?: "sm" | "md" | "lg";
  href?: string;
}

const variants = {
  primary:
    "bg-primary-600 hover:bg-primary-500 text-white shadow-lg shadow-primary-600/25 hover:-translate-y-0.5 hover:shadow-xl hover:shadow-primary-600/30 active:translate-y-0 active:shadow-lg button-glow-pulse",
  secondary:
    "border border-surface-600 hover:border-surface-400 text-surface-100 hover:bg-surface-800 hover:-translate-y-0.5 active:translate-y-0",
  ghost: "text-surface-400 hover:text-surface-100",
};

const sizes = {
  sm: "px-4 py-2 text-sm",
  md: "px-6 py-3 text-base",
  lg: "px-8 py-4 text-lg",
};

export function Button({
  variant = "primary",
  size = "md",
  href,
  children,
  className = "",
  ...props
}: ButtonProps) {
  const reduced = useReducedMotion();
  const magneticRef = useMagneticHover<HTMLAnchorElement>({ strength: 4 });
  const buttonMagneticRef = useMagneticHover<HTMLButtonElement>({ strength: 4 });
  const plainAnchorRef = useRef<HTMLAnchorElement>(null);
  const plainButtonRef = useRef<HTMLButtonElement>(null);

  const useMagnetic = variant === "primary" && !reduced;

  const classes = `inline-flex items-center justify-center font-medium rounded-lg transition-all duration-200 cursor-pointer ${variants[variant]} ${sizes[size]} ${className}`;

  if (href) {
    return (
      <a
        ref={useMagnetic ? magneticRef : plainAnchorRef}
        href={href}
        className={classes}
      >
        {children}
      </a>
    );
  }

  return (
    <button
      ref={useMagnetic ? buttonMagneticRef : plainButtonRef}
      className={classes}
      {...props}
    >
      {children}
    </button>
  );
}
