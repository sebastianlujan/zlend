import { content } from "../../data/content";
import { Badge } from "../ui/Badge";
import { SectionWrapper } from "../ui/SectionWrapper";
import { useScrollAnimation } from "../../hooks/useScrollAnimation";
import { useSplitTextHover } from "../../hooks/useSplitTextHover";

export function Solution() {
  const { solution } = content;
  const ref = useScrollAnimation<HTMLDivElement>({
    childSelector: "[data-animate]",
    staggerDelay: 0.15,
    animation: "fadeUp",
  });
  const headingRef = useSplitTextHover<HTMLHeadingElement>();

  return (
    <SectionWrapper id="solution">
      <div ref={ref}>
        <div className="text-center mb-16" data-animate>
          <Badge>{solution.sectionLabel}</Badge>
          <h2 ref={headingRef} className="section-heading mt-4 text-4xl md:text-5xl font-bold text-white">
            {solution.title}
          </h2>
          <p className="mt-4 text-lg text-surface-400 max-w-2xl mx-auto font-mono">
            {solution.subtitle}
          </p>
        </div>

        <div className="grid md:grid-cols-3 gap-8">
          {solution.steps.map((step, i) => (
            <div
              key={step.number}
              className="relative border-t-2 border-primary-700 bg-surface-900/50 rounded-lg p-6 hover:-translate-y-1 hover:shadow-lg hover:shadow-primary-900/20 transition-all duration-300"
              data-animate
            >
              {/* Terminal connector line between steps */}
              {i < solution.steps.length - 1 && (
                <div className="hidden md:block absolute top-1/2 -right-4 w-8 flow-line" />
              )}
              <div className="text-sm font-mono text-primary-500 mb-3">
                <span className="text-surface-600">{">"}</span> {step.number}_
              </div>
              <h3 className="text-2xl font-bold text-white mb-3">
                {step.title}
              </h3>
              <p className="text-surface-400 leading-relaxed font-mono text-sm">
                {step.description}
              </p>
            </div>
          ))}
        </div>
      </div>
    </SectionWrapper>
  );
}
