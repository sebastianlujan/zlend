import { useRef, useEffect } from "react";
import { gsap } from "../../lib/gsap";
import { animateCountUp } from "../../lib/animations";
import { useReducedMotion } from "../../hooks/useReducedMotion";

interface StatCardProps {
  value: string;
  unit?: string;
  label: string;
  subtext?: string;
  typedSubvalue?: string;
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

export function StatCard({ value, unit, label, subtext, typedSubvalue }: StatCardProps) {
  const numberRef = useRef<HTMLSpanElement>(null);
  const valueRowRef = useRef<HTMLDivElement>(null);
  const reduced = useReducedMotion();
  const parsed = parseValue(value);
  const fullPrimary = unit ? `${value} ${unit}` : value;

  // Count-up (only for non-alternating cards)
  useEffect(() => {
    const el = numberRef.current;
    if (!el || !parsed || reduced || typedSubvalue) return;

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
  }, [parsed, reduced, typedSubvalue]);

  // Alternating typewriter: "5.1M ZEC" ↔ "~$255M"
  useEffect(() => {
    const row = valueRowRef.current;
    if (!row || !typedSubvalue || reduced) return;

    let killed = false;
    const values = [fullPrimary, typedSubvalue];
    let current = 0;

    // Start showing the first value
    row.textContent = "";

    function typeText(text: string, onDone: () => void) {
      if (killed) return;
      const progress = { v: 0 };
      gsap.to(progress, {
        v: text.length,
        duration: text.length * 0.06,
        ease: "none",
        onUpdate() {
          row!.textContent = text.slice(0, Math.round(progress.v));
        },
        onComplete: onDone,
      });
    }

    function untype(text: string, onDone: () => void) {
      if (killed) return;
      const progress = { v: text.length };
      gsap.to(progress, {
        v: 0,
        duration: text.length * 0.03,
        ease: "none",
        onUpdate() {
          row!.textContent = text.slice(0, Math.round(progress.v));
        },
        onComplete: onDone,
      });
    }

    function cycle() {
      if (killed) return;
      const text = values[current];
      typeText(text, () => {
        if (killed) return;
        gsap.delayedCall(2.8, () => {
          if (killed) return;
          untype(text, () => {
            if (killed) return;
            current = (current + 1) % values.length;
            gsap.delayedCall(0.6, cycle);
          });
        });
      });
    }

    cycle();

    return () => {
      killed = true;
      gsap.killTweensOf(row);
    };
  }, [typedSubvalue, reduced, fullPrimary]);

  // Non-alternating render
  if (!typedSubvalue) {
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
        {subtext && <div className="mt-1 text-sm text-surface-400">{subtext}</div>}
      </div>
    );
  }

  // Alternating render — single line that swaps between values
  return (
    <div className="text-center p-6" data-animate>
      <div className="h-[3.5rem] flex items-center justify-center">
        <span
          ref={valueRowRef}
          className="text-4xl md:text-5xl font-bold text-white font-mono whitespace-nowrap cursor-blink"
        />
      </div>
      <div className="mt-2 text-surface-200 font-medium">{label}</div>
      {subtext && <div className="mt-1 text-sm text-surface-400">{subtext}</div>}
    </div>
  );
}
