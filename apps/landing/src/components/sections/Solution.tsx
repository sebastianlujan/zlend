import { content } from "../../data/content";
import { Badge } from "../ui/Badge";
import { SectionWrapper } from "../ui/SectionWrapper";
import { useScrollAnimation } from "../../hooks/useScrollAnimation";

export function Solution() {
  const { solution } = content;
  const ref = useScrollAnimation<HTMLDivElement>({ childSelector: "[data-animate]" });

  return (
    <SectionWrapper id="solution">
      <div ref={ref}>
        <div className="text-center mb-16" data-animate>
          <Badge>{solution.sectionLabel}</Badge>
          <h2 className="mt-4 text-4xl md:text-5xl font-bold text-white">
            {solution.title}
          </h2>
          <p className="mt-4 text-lg text-surface-400 max-w-2xl mx-auto">
            {solution.subtitle}
          </p>
        </div>

        <div className="grid md:grid-cols-3 gap-8">
          {solution.steps.map((step) => (
            <div key={step.number} className="relative" data-animate>
              <div className="text-7xl font-bold text-primary-900/50 font-mono mb-4">
                {step.number}
              </div>
              <h3 className="text-2xl font-bold text-white mb-3">
                {step.title}
              </h3>
              <p className="text-surface-400 leading-relaxed">
                {step.description}
              </p>
            </div>
          ))}
        </div>
      </div>
    </SectionWrapper>
  );
}
