import { content } from "../../data/content";
import { Badge } from "../ui/Badge";
import { SectionWrapper } from "../ui/SectionWrapper";
import { useScrollAnimation } from "../../hooks/useScrollAnimation";

export function Problem() {
  const { problem } = content;
  const introRef = useScrollAnimation<HTMLDivElement>({
    childSelector: "[data-animate]",
    animation: "fadeUp",
  });
  const inventoryRef = useScrollAnimation<HTMLDivElement>({
    childSelector: "[data-animate]",
    staggerDelay: 0.08,
    animation: "fadeUp",
  });
  const stablesRef = useScrollAnimation<HTMLDivElement>({
    childSelector: "[data-animate]",
  });
  const { zecInventory, withStables } = problem;

  return (
    <SectionWrapper id="problem">
      {/* Intro */}
      <div ref={introRef} className="text-center mb-24" data-animate>
        <Badge>{problem.sectionLabel}</Badge>
        <h2 className="section-heading mt-4 text-4xl md:text-5xl font-bold text-white">
          {problem.title}
        </h2>
        <p className="mt-4 text-lg text-surface-400 max-w-2xl mx-auto leading-relaxed">
          {problem.subtitle}
        </p>
      </div>

      {/* Scene 1: ZEC Inventory */}
      <div ref={inventoryRef} className="mb-32">
        <h3
          className="text-2xl md:text-3xl font-bold text-white mb-8 text-center"
          data-animate
        >
          {zecInventory.title}
        </h3>

        <div className="mx-auto max-w-md mb-2 font-mono text-xs text-primary-500/50" data-animate>
          {"// ZEC_INVENTORY"}
        </div>
        <div
          className="mx-auto max-w-md rounded-xl border border-surface-700/50 bg-surface-900/80 p-6 hover:border-surface-600/70 transition-colors duration-200"
          data-animate
        >
          {/* What you have */}
          <div className="space-y-2.5 mb-5 pb-5 border-b border-surface-700/50">
            {zecInventory.have.map((entry) => (
              <div
                key={entry.item}
                className="flex items-center gap-3"
                data-animate
              >
                <span className="text-primary-400 font-mono text-sm shrink-0">&#10003;</span>
                <span className="text-sm text-white font-medium">{entry.item}</span>
                <span className="text-xs text-surface-500 font-mono ml-auto">{entry.detail}</span>
              </div>
            ))}
          </div>

          {/* Opportunities */}
          <div className="space-y-2.5">
            {zecInventory.opportunities.map((item) => (
              <div
                key={item}
                className="flex items-center gap-3"
                data-animate
              >
                <span className="text-accent-400/60 font-mono text-sm shrink-0">&#8594;</span>
                <span className="text-sm text-surface-400">{item}</span>
              </div>
            ))}
          </div>

          {/* Note */}
          <div
            className="mt-5 pt-3 border-t border-surface-700/50 flex items-center gap-2"
            data-animate
          >
            <span className="text-xs text-surface-500 font-mono cursor-blink">
              {zecInventory.note}
            </span>
          </div>
        </div>
      </div>

      {/* Scene 2: ZEC Alone vs ZEC + Stables */}
      <div ref={stablesRef}>
        <h3
          className="text-2xl md:text-3xl font-bold text-white mb-8 text-center"
          data-animate
        >
          {withStables.title}
        </h3>

        <div className="grid md:grid-cols-2 gap-6 max-w-2xl mx-auto">
          {/* ZEC Alone */}
          <div
            className="rounded-xl border border-dashed border-surface-700/50 bg-surface-900/50 p-6 hover:border-surface-600 transition-colors duration-200"
            data-animate
          >
            <div className="text-xs font-mono uppercase tracking-wider text-surface-500 mb-4">
              {withStables.alone.label}
            </div>
            <ul className="space-y-2.5">
              {withStables.alone.items.map((item) => (
                <li
                  key={item}
                  className="flex items-center gap-2 text-sm text-surface-500"
                >
                  <span className="w-1 h-1 rounded-full bg-surface-600 shrink-0" />
                  {item}
                </li>
              ))}
            </ul>
          </div>

          {/* ZEC + Stables */}
          <div
            className="rounded-xl border border-primary-500/20 bg-surface-900/80 p-6 hover:border-primary-500/40 transition-colors duration-200"
            data-animate
          >
            <div className="text-xs font-mono uppercase tracking-wider text-primary-400 mb-4">
              {withStables.unlocked.label}
            </div>
            <ul className="space-y-2.5">
              {withStables.unlocked.items.map((item) => (
                <li
                  key={item}
                  className="flex items-center gap-2 text-sm text-surface-300"
                >
                  <span className="w-1 h-1 rounded-full bg-primary-400 shrink-0" />
                  {item}
                </li>
              ))}
            </ul>
          </div>
        </div>

        <p
          className="mt-10 text-center text-lg text-surface-300 italic max-w-xl mx-auto"
          data-animate
        >
          {withStables.closingLine}
        </p>
      </div>
    </SectionWrapper>
  );
}
