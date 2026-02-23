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

      const isMobile = window.matchMedia("(max-width: 1023px)").matches;

      const header = container.querySelector("[data-section-header]");
      const zcashPanel = container.querySelector("[data-world-panel='zcash']");
      const avaxPanel = container.querySelector("[data-world-panel='avalanche']");
      const bridge = container.querySelector("[data-bridge-column]");
      const steps = container.querySelectorAll("[data-journey-step]");

      // Header — always fadeUp
      if (header) gsap.set(header, { opacity: 0, y: 20 });

      // Panels — slide from sides on desktop, fadeUp on mobile
      if (zcashPanel) {
        gsap.set(zcashPanel, isMobile ? { opacity: 0, y: 20 } : { opacity: 0, x: -30 });
      }
      if (avaxPanel) {
        gsap.set(avaxPanel, isMobile ? { opacity: 0, y: 20 } : { opacity: 0, x: 30 });
      }

      // Bridge — scale from zero
      if (bridge) gsap.set(bridge, { opacity: 0, scale: 0, transformOrigin: "center center" });

      // Journey steps — fadeUp
      gsap.set(steps, { opacity: 0, y: 20 });

      const tl = gsap.timeline({
        scrollTrigger: {
          trigger: container,
          start: "top 80%",
          toggleActions: "play none none none",
        },
      });

      // 1. Header fades up
      if (header) {
        tl.to(header, {
          opacity: 1, y: 0, duration: 0.5, ease: "power3.out",
          onStart() { container.classList.add("in-view"); },
        });
      }

      // 2. Panels slide in from opposite sides
      if (zcashPanel) {
        tl.to(zcashPanel, {
          opacity: 1, x: 0, y: 0, duration: 0.6, ease: "power3.out",
        }, "-=0.1");
      }
      if (avaxPanel) {
        tl.to(avaxPanel, {
          opacity: 1, x: 0, y: 0, duration: 0.6, ease: "power3.out",
        }, "-=0.5");
      }

      // 3. Bridge appears with scale + glow pulse (0.3s delay for anticipation)
      if (bridge) {
        tl.to(bridge, {
          opacity: 1,
          scale: 1,
          duration: 0.5,
          ease: "back.out(1.4)",
        }, "+=0.3");
        // Glow pulse effect
        tl.to(bridge, {
          boxShadow: "0 0 30px rgba(232,65,66,0.3), 0 0 60px rgba(232,65,66,0.1)",
          duration: 0.4,
          ease: "power2.out",
        }, "-=0.2");
        tl.to(bridge, {
          boxShadow: "0 0 0px rgba(232,65,66,0), 0 0 0px rgba(232,65,66,0)",
          duration: 0.6,
          ease: "power2.in",
        });
      }

      // 5. Journey steps cascade with stagger
      tl.to(steps, {
        opacity: 1, y: 0, duration: 0.5, stagger: 0.12, ease: "power3.out",
      }, "-=0.3");
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
