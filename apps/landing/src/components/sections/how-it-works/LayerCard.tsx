import { useState } from "react";
import type { Content } from "../../../data/content";
import { LayerDetail } from "./LayerDetail";

type Layer = Content["howItWorks"]["layers"][number];

const colorMap: Record<string, { border: string; dot: string; stepColor: string }> = {
  gold: {
    border: "border-[#F4B728]/20 hover:border-[#F4B728]/40",
    dot: "bg-[#F4B728]",
    stepColor: "text-[#F4B728]",
  },
  accent: {
    border: "border-accent-500/20 hover:border-accent-500/40",
    dot: "bg-accent-500",
    stepColor: "text-accent-500",
  },
  primary: {
    border: "border-primary-500/20 hover:border-primary-500/40",
    dot: "bg-primary-500",
    stepColor: "text-primary-500",
  },
};

interface LayerCardProps {
  layer: Layer;
}

export function LayerCard({ layer }: LayerCardProps) {
  const [expanded, setExpanded] = useState(false);
  const colors = colorMap[layer.color] ?? colorMap.primary;

  return (
    <div
      className={`relative rounded-xl border bg-surface-900/50 backdrop-blur-sm p-5 md:p-6 transition-colors duration-200 cursor-pointer ${colors.border}`}
      onClick={() => setExpanded(!expanded)}
      data-layer-card={layer.id}
    >
      <div className="flex items-start gap-4">
        {/* Step indicator */}
        <div className="flex flex-col items-center shrink-0">
          <span className={`text-xs font-mono font-bold ${colors.stepColor}`}>
            {layer.step}
          </span>
          <span className={`w-2 h-2 rounded-full mt-1 ${colors.dot}`} />
        </div>

        {/* Content */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 flex-wrap">
            <h3 className="text-base font-bold text-white">{layer.label}</h3>
            <span className="text-[10px] font-mono text-surface-500 uppercase tracking-widest px-2 py-0.5 rounded-full border border-surface-700/50 bg-surface-900/50">
              {layer.network}
            </span>
          </div>
          <p className="text-sm text-surface-400 font-mono mt-1">
            {layer.description}
          </p>

          {/* Expandable detail */}
          <LayerDetail
            detail={layer.detail}
            terminalLine={layer.terminalLine}
            expanded={expanded}
          />

          {/* Expand indicator */}
          <div className="mt-2 flex items-center gap-1 text-[10px] font-mono text-surface-600">
            <span>{expanded ? "−" : "+"}</span>
            <span>{expanded ? "less" : "terminal"}</span>
          </div>
        </div>
      </div>
    </div>
  );
}
