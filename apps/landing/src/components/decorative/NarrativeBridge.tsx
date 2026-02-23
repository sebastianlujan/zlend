import { useRef } from "react";
import { useGSAP } from "@gsap/react";
import { gsap } from "../../lib/gsap";
import { useReducedMotion } from "../../hooks/useReducedMotion";

interface NarrativeBridgeProps {
  prompt: string;
}

export function NarrativeBridge({ prompt }: NarrativeBridgeProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const svgRef = useRef<SVGSVGElement>(null);
  const textRef = useRef<HTMLParagraphElement>(null);
  const chevronRef = useRef<SVGSVGElement>(null);
  const reduced = useReducedMotion();

  useGSAP(
    () => {
      if (reduced) return;
      const path = svgRef.current?.querySelector("[data-bridge-path]") as SVGPathElement | null;
      const text = textRef.current;
      const chevron = chevronRef.current;

      if (path) {
        const len = path.getTotalLength();
        gsap.set(path, { strokeDasharray: len, strokeDashoffset: len });
        gsap.to(path, {
          strokeDashoffset: 0,
          duration: 0.8,
          ease: "power3.out",
          scrollTrigger: {
            trigger: containerRef.current,
            start: "top 85%",
            toggleActions: "play none none none",
          },
        });
      }

      if (text) {
        gsap.set(text, { opacity: 0, y: 10 });
        gsap.to(text, {
          opacity: 1,
          y: 0,
          duration: 0.5,
          ease: "power3.out",
          scrollTrigger: {
            trigger: containerRef.current,
            start: "top 80%",
            toggleActions: "play none none none",
          },
        });
      }

      if (chevron) {
        gsap.set(chevron, { opacity: 0 });
        gsap.to(chevron, {
          opacity: 1,
          duration: 0.3,
          delay: 0.6,
          scrollTrigger: {
            trigger: containerRef.current,
            start: "top 80%",
            toggleActions: "play none none none",
          },
        });
      }
    },
    { scope: containerRef },
  );

  return (
    <div
      ref={containerRef}
      className="relative flex flex-col items-center py-12 md:py-16"
      aria-hidden="true"
    >
      {/* SVG vertical line */}
      <svg
        ref={svgRef}
        width="2"
        height="48"
        viewBox="0 0 2 48"
        fill="none"
        className="mb-4"
      >
        <path
          data-bridge-path
          d="M1 0 V48"
          stroke="rgba(156,156,166,0.25)"
          strokeWidth="1"
          strokeDasharray="4 4"
        />
      </svg>

      {/* Curiosity prompt */}
      <p
        ref={textRef}
        className="text-sm md:text-base font-mono text-surface-400 text-center max-w-md px-6 cursor-blink"
      >
        {prompt}
      </p>

      {/* Pulsing chevron */}
      <svg
        ref={chevronRef}
        width="16"
        height="16"
        viewBox="0 0 16 16"
        fill="none"
        className="mt-4 animate-bounce"
      >
        <path
          d="M4 6 L8 10 L12 6"
          stroke="rgba(156,156,166,0.4)"
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
    </div>
  );
}
