import { content } from "../../data/content";
import { Badge } from "../ui/Badge";
import { Card } from "../ui/Card";
import { SectionWrapper } from "../ui/SectionWrapper";
import { useScrollAnimation } from "../../hooks/useScrollAnimation";

export function HowItWorks() {
  const { howItWorks } = content;
  const ref = useScrollAnimation<HTMLDivElement>({ childSelector: "[data-animate]" });

  return (
    <SectionWrapper id="how-it-works" className="bg-surface-900/30">
      <div ref={ref}>
        <div className="text-center mb-16" data-animate>
          <Badge>{howItWorks.sectionLabel}</Badge>
          <h2 className="mt-4 text-4xl md:text-5xl font-bold text-white">
            {howItWorks.title}
          </h2>
        </div>

        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {howItWorks.features.map((feature) => (
            <Card key={feature.title}>
              <div data-animate>
                <h3 className="text-lg font-semibold text-white mb-2">
                  {feature.title}
                </h3>
                <p className="text-surface-400 text-sm leading-relaxed">
                  {feature.description}
                </p>
              </div>
            </Card>
          ))}
        </div>
      </div>
    </SectionWrapper>
  );
}
