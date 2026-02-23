import type { Content } from "../../../data/content";

type Step = Content["solution"]["steps"][number];

interface JourneyStepProps {
  step: Step;
}

const sideStyles: Record<string, { border: string; dot: string; stepColor: string; networkLabel: string }> = {
  zcash: {
    border: "border-[#F4B728]/20 hover:border-[#F4B728]/40",
    dot: "bg-[#F4B728]",
    stepColor: "text-[#F4B728]",
    networkLabel: "Zcash",
  },
  bridge: {
    border: "border-surface-600/20 hover:border-surface-500/40",
    dot: "bg-surface-400",
    stepColor: "text-surface-400",
    networkLabel: "OGBank",
  },
  avalanche: {
    border: "border-primary-500/20 hover:border-primary-500/40",
    dot: "bg-primary-500",
    stepColor: "text-primary-500",
    networkLabel: "Avalanche",
  },
};

export function JourneyStep({ step }: JourneyStepProps) {
  const colors = sideStyles[step.side] ?? sideStyles.bridge;

  return (
    <div
      className={`group relative rounded-xl border bg-surface-900/50 backdrop-blur-sm p-5 md:p-6 transition-colors duration-200 ${colors.border}`}
      data-journey-step
    >
      <div className="flex items-start gap-4">
        {/* Step indicator with colored dot */}
        <div className="flex flex-col items-center shrink-0">
          <span className={`text-xs font-mono font-bold ${colors.stepColor}`}>
            {step.number}
          </span>
          <span className={`w-2 h-2 rounded-full mt-1 ${colors.dot}`} />
        </div>

        {/* Content */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 flex-wrap">
            <h3 className="text-xl md:text-2xl font-bold text-white font-mono leading-tight">
              {step.title}
            </h3>
            <span className="text-[10px] font-mono text-surface-500 uppercase tracking-widest px-2 py-0.5 rounded-full border border-surface-700/50 bg-surface-900/50">
              {colors.networkLabel}
            </span>
          </div>

          <p className="text-sm text-surface-400 font-mono mt-2 leading-relaxed">
            {step.description}
          </p>

          <span className="inline-block mt-3 text-[10px] font-mono text-surface-600 group-hover:text-surface-400 transition-colors duration-200">
            {">"} step_{step.number}_
          </span>
        </div>
      </div>
    </div>
  );
}
