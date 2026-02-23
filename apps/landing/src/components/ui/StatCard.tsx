import { useRef, useEffect } from "react";
import { animateCountUp } from "../../lib/animations";
import { useReducedMotion } from "../../hooks/useReducedMotion";

interface StatCardProps {
  value: string;
  unit?: string;
  label: string;
  subtext?: string;
}

/** Parse "5.1M" → { prefix: "", number: 5.1, suffix: "M" }. Skip ranges like "10-12K". */
function parseValue(value: string) {
  if (value.includes("-")) return null;
  const match = value.match(/^([^0-9]*)([0-9.]+)(.*)$/);
  if (!match) return null;
  const num = parseFloat(match[2]);
  if (isNaN(num) || num === 0) return null;
  return { prefix: match[1], number: num, suffix: match[3] };
}

export function StatCard({ value, unit, label, subtext }: StatCardProps) {
  const numberRef = useRef<HTMLSpanElement>(null);
  const reduced = useReducedMotion();
  const parsed = parseValue(value);

  useEffect(() => {
    const el = numberRef.current;
    if (!el || !parsed || reduced) return;

    el.textContent = "0";

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (!entry.isIntersecting) return;
        animateCountUp(el, parsed.number);
        observer.unobserve(el);
      },
      { threshold: 0.3 },
    );

    observer.observe(el);
    return () => observer.disconnect();
  }, [parsed, reduced]);

  return (
    <div className="text-center p-6" data-animate>
      <div className="text-4xl md:text-5xl font-bold text-white font-mono">
        {parsed ? (
          <>
            {parsed.prefix}
            <span ref={numberRef}>{parsed.number}</span>
            {parsed.suffix}
          </>
        ) : (
          value
        )}
        {unit && <span className="text-primary-400 ml-1 text-2xl md:text-3xl">{unit}</span>}
      </div>
      <div className="mt-2 text-surface-200 font-medium">{label}</div>
      {subtext && (
        <div className="mt-1 text-sm text-surface-400">{subtext}</div>
      )}
    </div>
  );
}
