import { useRef } from "react";
import { useGSAP } from "@gsap/react";
import { content } from "../../data/content";
import { gsap, ScrollTrigger } from "../../lib/gsap";
import { useReducedMotion } from "../../hooks/useReducedMotion";
import { useSplitTextHover } from "../../hooks/useSplitTextHover";
import { Badge } from "../ui/Badge";
import { SectionWrapper } from "../ui/SectionWrapper";
import { PipelineNode } from "./how-it-works/PipelineNode";
import { PipelineConnector } from "./how-it-works/PipelineConnector";
import { TechHighlights } from "./how-it-works/TechHighlights";

export function HowItWorks() {
  const { howItWorks } = content;
  const containerRef = useRef<HTMLDivElement>(null);
  const reduced = useReducedMotion();
  const headingRef = useSplitTextHover<HTMLHeadingElement>();

  useGSAP(
    () => {
      if (reduced) return;
      const container = containerRef.current;
      if (!container) return;

      const nodes = container.querySelectorAll("[data-pipeline-node]");
      const connectorPaths = container.querySelectorAll("[data-connector-path]");
      const particles = container.querySelectorAll("[data-connector-particle]");
      const badges = container.querySelectorAll("[data-tech-badge]");
      const header = container.querySelector("[data-section-header]");

      gsap.set(nodes, { opacity: 0, scale: 0.85, y: 20 });
      gsap.set(badges, { opacity: 0, y: 10 });
      if (header) gsap.set(header, { opacity: 0, y: 30 });

      connectorPaths.forEach((path) => {
        const svgPath = path as SVGPathElement;
        const length = svgPath.getTotalLength();
        gsap.set(svgPath, { strokeDasharray: length, strokeDashoffset: length });
      });
      gsap.set(particles, { opacity: 0 });

      ScrollTrigger.matchMedia({
        // Desktop & mobile use the same timeline, just different trigger range
        all: () => {
          const tl = gsap.timeline({
            scrollTrigger: {
              trigger: container,
              start: "top 78%",
              end: "bottom 60%",
              scrub: 1.2,
            },
          });

          // Header
          if (header) {
            tl.to(header, {
              opacity: 1,
              y: 0,
              duration: 0.3,
              ease: "power3.out",
            });
          }

          // Interleave nodes and connectors
          nodes.forEach((node, i) => {
            tl.to(node, {
              opacity: 1,
              scale: 1,
              y: 0,
              duration: 0.3,
              ease: "power3.out",
            });

            if (i < connectorPaths.length) {
              const svgPath = connectorPaths[i] as SVGPathElement;
              const pathLength = svgPath.getTotalLength();
              const particle = particles[i] as SVGCircleElement;

              // Draw connector line
              tl.to(
                svgPath,
                {
                  strokeDashoffset: 0,
                  duration: 0.3,
                  ease: "power2.inOut",
                },
                "-=0.1",
              );

              // Animate particle along path using a proxy element for progress tracking
              const particleTween = gsap.fromTo(
                particle,
                { opacity: 1 },
                {
                  opacity: 1,
                  duration: 0.25,
                  ease: "power1.inOut",
                  paused: true,
                  onUpdate() {
                    const ratio = particleTween.ratio;
                    const point = svgPath.getPointAtLength(ratio * pathLength);
                    particle.setAttribute("cx", String(point.x));
                    particle.setAttribute("cy", String(point.y));
                  },
                  onComplete() {
                    gsap.to(particle, { opacity: 0, duration: 0.15 });
                  },
                },
              );
              tl.add(particleTween.play(), "-=0.2");
            }
          });

          // Tech badges
          tl.to(badges, {
            opacity: 1,
            y: 0,
            duration: 0.2,
            stagger: 0.05,
            ease: "power2.out",
          });
        },
      });
    },
    { scope: containerRef, dependencies: [reduced] },
  );

  const connectorColors: Array<"primary" | "accent"> = [
    "primary",
    "accent",
    "primary",
  ];

  return (
    <SectionWrapper id="how-it-works" className="bg-surface-900/30">
      <div ref={containerRef}>
        {/* Header */}
        <div className="text-center mb-16" data-section-header>
          <Badge>{howItWorks.sectionLabel}</Badge>
          <h2
            ref={headingRef}
            className="section-heading mt-4 text-4xl md:text-5xl font-bold text-white"
          >
            {howItWorks.title}
          </h2>
          <p className="mt-4 text-lg text-surface-400 max-w-2xl mx-auto font-mono">
            {howItWorks.subtitle}
          </p>
        </div>

        {/* Pipeline — horizontal on lg+, vertical below */}
        <div className="hidden lg:flex items-start justify-center gap-0 max-w-5xl mx-auto">
          {howItWorks.pipeline.map((step, i) => (
            <div key={step.id} className="contents">
              <PipelineNode step={step} />
              {i < howItWorks.pipeline.length - 1 && (
                <PipelineConnector
                  orientation="horizontal"
                  color={connectorColors[i]}
                  className="mt-8"
                />
              )}
            </div>
          ))}
        </div>

        {/* Pipeline — vertical on mobile/tablet */}
        <div className="flex lg:hidden flex-col items-center gap-0">
          {howItWorks.pipeline.map((step, i) => (
            <div key={step.id} className="contents">
              <PipelineNode step={step} />
              {i < howItWorks.pipeline.length - 1 && (
                <PipelineConnector
                  orientation="vertical"
                  color={connectorColors[i]}
                  className="my-4"
                />
              )}
            </div>
          ))}
        </div>

        {/* Tech highlights */}
        <TechHighlights highlights={howItWorks.highlights} />
      </div>
    </SectionWrapper>
  );
}
