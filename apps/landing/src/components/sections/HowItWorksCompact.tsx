import { content } from "../../data/content";
import { Badge } from "../ui/Badge";
import { SectionWrapper } from "../ui/SectionWrapper";
import { useScrollAnimation } from "../../hooks/useScrollAnimation";

const colorMap: Record<string, { border: string; dot: string; step: string }> = {
  gold: {
    border: "border-[#F4B728]/20 hover:border-[#F4B728]/40",
    dot: "bg-[#F4B728]",
    step: "text-[#F4B728]",
  },
  accent: {
    border: "border-accent-500/20 hover:border-accent-500/40",
    dot: "bg-accent-500",
    step: "text-accent-500",
  },
  primary: {
    border: "border-primary-500/20 hover:border-primary-500/40",
    dot: "bg-primary-500",
    step: "text-primary-500",
  },
};

export function HowItWorksCompact() {
  const { howItWorks } = content;
  const ref = useScrollAnimation<HTMLDivElement>({
    childSelector: "[data-animate]",
    animation: "fadeUp",
  });

  return (
    <SectionWrapper id="how-it-works" className="bg-surface-900/30">
      <div ref={ref}>
        <div className="text-center mb-12" data-animate>
          <Badge>{howItWorks.sectionLabel}</Badge>
          <h2 className="section-heading mt-4 text-4xl md:text-5xl font-bold text-white">
            {howItWorks.title}
          </h2>
          <p className="mt-4 text-lg text-surface-400 max-w-2xl mx-auto font-mono">
            {howItWorks.subtitle}
          </p>
        </div>

        <div
          className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 max-w-5xl mx-auto"
          data-animate
        >
          {howItWorks.layers.map((layer) => {
            const colors = colorMap[layer.color] ?? colorMap.primary;
            return (
              <div
                key={layer.id}
                className={`rounded-xl border bg-surface-900/50 backdrop-blur-sm p-4 transition-colors duration-200 ${colors.border}`}
              >
                <span className={`text-xs font-mono font-bold ${colors.step}`}>
                  {layer.step}
                </span>
                <h3 className="text-lg font-bold text-white font-mono mt-2 leading-tight">
                  {layer.label}
                </h3>
                <span className="inline-block mt-2 text-[10px] font-mono text-surface-500 uppercase tracking-widest px-2 py-0.5 rounded-full border border-surface-700/50 bg-surface-900/50">
                  {layer.network}
                </span>
                <p className="text-sm text-surface-400 font-mono mt-2">
                  {layer.description}
                </p>
              </div>
            );
          })}
        </div>

        <div className="text-center mt-8" data-animate>
          <a
            href="/technology"
            className="text-sm font-mono text-surface-500 hover:text-white transition-colors duration-200"
          >
            {">"} full_architecture →
          </a>
        </div>
      </div>
    </SectionWrapper>
  );
}
