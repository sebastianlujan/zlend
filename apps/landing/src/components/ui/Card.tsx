import { useRef, useCallback } from "react";
import { useTiltHover } from "../../hooks/useTiltHover";
import { useReducedMotion } from "../../hooks/useReducedMotion";

interface CardProps {
  children: React.ReactNode;
  className?: string;
}

export function Card({ children, className = "" }: CardProps) {
  const tiltRef = useTiltHover<HTMLDivElement>();
  const reduced = useReducedMotion();
  const glowRef = useRef<HTMLDivElement>(null);

  const onMouseMove = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      if (reduced) return;
      const el = glowRef.current;
      if (!el) return;
      const rect = el.getBoundingClientRect();
      el.style.setProperty("--mouse-x", `${e.clientX - rect.left}px`);
      el.style.setProperty("--mouse-y", `${e.clientY - rect.top}px`);
    },
    [reduced],
  );

  return (
    <div
      ref={(node) => {
        glowRef.current = node;
        // Assign to tilt ref too
        if (typeof tiltRef === "object" && tiltRef !== null) {
          (tiltRef as React.MutableRefObject<HTMLDivElement | null>).current = node;
        }
      }}
      onMouseMove={onMouseMove}
      className={`card-glow relative rounded-xl border border-surface-700/50 bg-surface-900/50 backdrop-blur-sm p-6 transition-all duration-300 hover:border-primary-700/40 hover:-translate-y-1 hover:shadow-lg hover:shadow-primary-900/20 hover:bg-surface-900/70 ${className}`}
    >
      {children}
    </div>
  );
}
