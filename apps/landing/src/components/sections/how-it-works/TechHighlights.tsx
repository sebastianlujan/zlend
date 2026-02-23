import type { Content } from "../../../data/content";

type Highlight = Content["howItWorks"]["highlights"][number];

export function TechHighlights({ highlights }: { highlights: readonly Highlight[] }) {
  return (
    <div className="flex flex-wrap justify-center gap-3 mt-16">
      {highlights.map((h) => (
        <span
          key={h.label}
          className="inline-flex items-center gap-2 px-4 py-2 text-xs font-mono text-surface-300 border border-surface-700/50 rounded-full bg-surface-900/50 hover:border-primary-700/30 transition-colors"
          data-tech-badge
        >
          <span className="w-1.5 h-1.5 rounded-full bg-primary-500/60" />
          {h.label}
        </span>
      ))}
    </div>
  );
}
