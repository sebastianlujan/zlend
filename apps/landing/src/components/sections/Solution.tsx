import { useRef } from "react";
import { useGSAP } from "@gsap/react";
import { content } from "../../data/content";
import { gsap } from "../../lib/gsap";
import { useReducedMotion } from "../../hooks/useReducedMotion";
import { Badge } from "../ui/Badge";
import { SectionWrapper } from "../ui/SectionWrapper";
import { WorldPanel } from "./solution/WorldPanel";
import { BridgeColumn } from "./solution/BridgeColumn";
import { JourneyStep } from "./solution/JourneyStep";

export function Solution() {
  const { solution } = content;
  const containerRef = useRef<HTMLDivElement>(null);
  const reduced = useReducedMotion();

  useGSAP(
    () => {
      if (reduced) return;
      const container = containerRef.current;
      if (!container) return;

      const header = container.querySelector("[data-section-header]");
      const zcashPanel = container.querySelector("[data-world-panel='zcash']");
      const avaxPanel = container.querySelector("[data-world-panel='avalanche']");
      const bridge = container.querySelector("[data-bridge-column]");
      const steps = container.querySelectorAll("[data-journey-step]");

      // Unified initial state: fadeUp
      const hidden = { opacity: 0, y: 20 };
      const visible = { opacity: 1, y: 0, duration: 0.5, ease: "power3.out" };

      if (header) gsap.set(header, hidden);
      if (zcashPanel) gsap.set(zcashPanel, hidden);
      if (avaxPanel) gsap.set(avaxPanel, hidden);
      if (bridge) gsap.set(bridge, hidden);
      gsap.set(steps, hidden);

      const tl = gsap.timeline({
        scrollTrigger: {
          trigger: container,
          start: "top 80%",
          toggleActions: "play none none none",
        },
      });

      if (header) {
        tl.to(header, {
          ...visible,
          onStart() { container.classList.add("in-view"); },
        });
      }

      if (zcashPanel) tl.to(zcashPanel, visible, "-=0.2");
      if (avaxPanel) tl.to(avaxPanel, visible, "-=0.3");
      if (bridge) tl.to(bridge, visible, "-=0.2");

      tl.to(steps, { ...visible, stagger: 0.1 }, "-=0.2");
    },
    { scope: containerRef, dependencies: [reduced] },
  );

  return (
    <SectionWrapper id="solution">
      <div ref={containerRef}>
        {/* Header */}
        <div className="text-center mb-16" data-section-header>
          <Badge>{solution.sectionLabel}</Badge>
          <h2 className="section-heading mt-4 text-4xl md:text-5xl font-bold text-white">
            {solution.title}
          </h2>
          <p className="mt-4 text-lg text-surface-400 max-w-2xl mx-auto font-mono">
            {solution.subtitle}
          </p>
        </div>

        {/* Split-world layout */}
        {/* Desktop: 3-column grid */}
        <div className="hidden lg:grid lg:grid-cols-[1fr_auto_1fr] lg:gap-8 lg:items-center lg:max-w-5xl lg:mx-auto">
          <WorldPanel world={solution.worlds.zcash} side="zcash" />
          <BridgeColumn bridge={solution.bridge} />
          <WorldPanel world={solution.worlds.avalanche} side="avalanche" />
        </div>

        {/* Tablet */}
        <div className="hidden md:flex lg:hidden flex-col items-center gap-6 max-w-lg mx-auto">
          <div className="grid grid-cols-2 gap-4 w-full">
            <WorldPanel world={solution.worlds.zcash} side="zcash" />
            <WorldPanel world={solution.worlds.avalanche} side="avalanche" />
          </div>
          <BridgeColumn bridge={solution.bridge} />
        </div>

        {/* Mobile: vertical stack */}
        <div className="md:hidden flex flex-col items-center gap-6">
          <WorldPanel world={solution.worlds.zcash} side="zcash" />
          <BridgeColumn bridge={solution.bridge} />
          <WorldPanel world={solution.worlds.avalanche} side="avalanche" />
        </div>

        {/* Journey Steps */}
        <div className="mt-16 grid md:grid-cols-3 gap-5 max-w-5xl mx-auto">
          {solution.steps.map((step) => (
            <JourneyStep key={step.number} step={step} />
          ))}
        </div>
      </div>
    </SectionWrapper>
  );
}
