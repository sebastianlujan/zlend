import { content } from "../../data/content";
import { Badge } from "../ui/Badge";
import { StatCard } from "../ui/StatCard";
import { SectionWrapper } from "../ui/SectionWrapper";
import { useScrollAnimation } from "../../hooks/useScrollAnimation";

export function MarketData() {
  const { market } = content;
  const ref = useScrollAnimation<HTMLDivElement>({
    childSelector: "[data-animate]",
    animation: "fadeUp",
  });
  return (
    <SectionWrapper id="market" className="bg-surface-900/30">
      <div ref={ref}>
        <div className="text-center mb-16" data-animate>
          <Badge>{market.sectionLabel}</Badge>
          <h2 className="section-heading mt-4 text-4xl md:text-5xl font-bold text-white">
            {market.title}
          </h2>
        </div>

        <div className="grid grid-cols-2 lg:grid-cols-4 gap-6">
          {market.stats.map((stat) => (
            <StatCard
              key={stat.label}
              value={stat.value}
              unit={stat.unit}
              label={stat.label}
              subtext={stat.subtext}
              typedSubvalue={"typedSubvalue" in stat ? stat.typedSubvalue : undefined}
            />
          ))}
        </div>

        <div className="mt-8 text-center" data-animate>
          <a
            href={market.source.href}
            target="_blank"
            rel="noopener noreferrer"
            className="text-sm text-surface-500 hover:text-primary-400 transition-colors underline underline-offset-4"
          >
            {market.source.label} ↗
          </a>
        </div>
      </div>
    </SectionWrapper>
  );
}
