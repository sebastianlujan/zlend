import { useEffect, useRef } from "react";
import { gsap, ScrollTrigger } from "../../lib/gsap";
import { useReducedMotion } from "../../hooks/useReducedMotion";

export function ScrollProgress() {
  const barRef = useRef<HTMLDivElement>(null);
  const reduced = useReducedMotion();

  useEffect(() => {
    const bar = barRef.current;
    if (!bar || reduced) return;

    gsap.to(bar, {
      scaleX: 1,
      ease: "none",
      scrollTrigger: {
        trigger: document.documentElement,
        start: "top top",
        end: "bottom bottom",
        scrub: 0.3,
      },
    });

    return () => {
      ScrollTrigger.getAll()
        .filter((st) => st.vars.trigger === document.documentElement)
        .forEach((st) => st.kill());
    };
  }, [reduced]);

  if (reduced) return null;

  return (
    <div
      ref={barRef}
      className="fixed top-0 left-0 right-0 h-0.5 bg-primary-500 z-[60] origin-left"
      style={{ transform: "scaleX(0)" }}
      aria-hidden="true"
    />
  );
}
