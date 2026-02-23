import { useRef } from "react";
import { useGSAP } from "@gsap/react";
import { content } from "../../data/content";
import { gsap } from "../../lib/gsap";
import { useReducedMotion } from "../../hooks/useReducedMotion";
import { Badge } from "../ui/Badge";
import { SectionWrapper } from "../ui/SectionWrapper";
import { LayerCard } from "./how-it-works/LayerCard";
import { DataFlowSpine } from "./how-it-works/DataFlowSpine";
import { TechHighlights } from "./how-it-works/TechHighlights";

export function HowItWorks() {
  const { howItWorks } = content;
  const containerRef = useRef<HTMLDivElement>(null);
  const reduced = useReducedMotion();

  useGSAP(
    () => {
      if (reduced) return;
      const container = containerRef.current;
      if (!container) return;

      const header = container.querySelector("[data-section-header]");
      const cards = container.querySelectorAll("[data-layer-card]");
      const badges = container.querySelectorAll("[data-tech-badge]");

      const hidden = { opacity: 0, y: 20 };

      if (header) gsap.set(header, hidden);
      gsap.set(cards, hidden);
      gsap.set(badges, hidden);

      // Header
      if (header) {
        gsap.to(header, {
          opacity: 1, y: 0, duration: 0.5, ease: "power3.out",
          scrollTrigger: {
            trigger: container,
            start: "top 80%",
            toggleActions: "play none none none",
          },
        });
      }

      // Layer cards — staggered fadeUp
      const cardContainer = container.querySelector("[data-layer-stack]");
      if (cardContainer && cards.length > 0) {
        gsap.to(cards, {
          opacity: 1, y: 0, duration: 0.5, stagger: 0.1, ease: "power3.out",
          scrollTrigger: {
            trigger: cardContainer,
            start: "top 80%",
            toggleActions: "play none none none",
          },
          onStart() { container.classList.add("in-view"); },
        });
      }

      // Tech badges
      if (badges.length > 0) {
        gsap.to(badges, {
          opacity: 1, y: 0, duration: 0.5, stagger: 0.04, ease: "power3.out",
          scrollTrigger: {
            trigger: badges[0].parentElement,
            start: "top 90%",
            toggleActions: "play none none none",
          },
        });
      }
    },
    { scope: containerRef, dependencies: [reduced] },
  );

  return (
    <SectionWrapper id="how-it-works" className="bg-surface-900/30">
      <div ref={containerRef}>
        {/* Header */}
        <div className="text-center mb-16" data-section-header>
          <Badge>{howItWorks.sectionLabel}</Badge>
          <h2 className="section-heading mt-4 text-4xl md:text-5xl font-bold text-white">
            {howItWorks.title}
          </h2>
          <p className="mt-4 text-lg text-surface-400 max-w-2xl mx-auto font-mono">
            {howItWorks.subtitle}
          </p>
        </div>

        {/* Layer stack with spine */}
        <div className="relative max-w-4xl mx-auto">
          <DataFlowSpine layerCount={howItWorks.layers.length} />

          <div
            className="flex flex-col gap-4 lg:pl-10"
            data-layer-stack
          >
            {howItWorks.layers.map((layer) => (
              <LayerCard key={layer.id} layer={layer} />
            ))}
          </div>
        </div>

        {/* Tech highlights */}
        <TechHighlights highlights={howItWorks.highlights} />
      </div>
    </SectionWrapper>
  );
}
