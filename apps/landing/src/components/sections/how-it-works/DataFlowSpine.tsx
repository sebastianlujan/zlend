import { useRef } from "react";
import { useGSAP } from "@gsap/react";
import { gsap } from "../../../lib/gsap";
import { useReducedMotion } from "../../../hooks/useReducedMotion";

interface DataFlowSpineProps {
  layerCount: number;
}

export function DataFlowSpine({ layerCount }: DataFlowSpineProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const reduced = useReducedMotion();

  useGSAP(
    () => {
      if (reduced || !svgRef.current) return;
      const svg = svgRef.current;
      const path = svg.querySelector("[data-spine-path]") as SVGPathElement | null;

      if (!path) return;

      const pathLength = path.getTotalLength();

      gsap.set(path, { strokeDasharray: pathLength, strokeDashoffset: pathLength });

      gsap.to(path, {
        strokeDashoffset: 0,
        duration: 1,
        ease: "power3.out",
        scrollTrigger: {
          trigger: svg,
          start: "top 80%",
          toggleActions: "play none none none",
        },
      });
    },
    { scope: svgRef },
  );

  const height = layerCount * 120;

  return (
    <svg
      ref={svgRef}
      width="24"
      height={height}
      viewBox={`0 0 24 ${height}`}
      fill="none"
      className="hidden lg:block absolute left-0 top-0"
      style={{ height: `${height}px` }}
    >
      <path
        data-spine-path
        d={`M12 0 V${height}`}
        stroke="rgba(156,156,166,0.2)"
        strokeWidth="1"
        strokeDasharray="4 4"
      />
    </svg>
  );
}
