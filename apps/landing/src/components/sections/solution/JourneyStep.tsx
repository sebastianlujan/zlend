import type { Content } from "../../../data/content";

type Step = Content["solution"]["steps"][number];

interface JourneyStepProps {
  step: Step;
}

export function JourneyStep({ step }: JourneyStepProps) {
  const sideColors: Record<string, string> = {
    zcash: "border-[#F4B728]/30",
    bridge: "border-surface-500/30",
    avalanche: "border-primary-500/30",
  };

  return (
    <div
      className={`relative rounded-xl border bg-surface-900/50 backdrop-blur-sm p-5 ${sideColors[step.side]}`}
      data-journey-step
    >
      <span className="text-xs font-mono text-surface-500">
        <span className="text-surface-600">{">"}</span> {step.number}_
      </span>
      <h3 className="text-lg font-bold text-white mt-1">{step.title}</h3>
      <p className="text-sm text-surface-400 font-mono mt-2 leading-relaxed">
        {step.description}
      </p>
    </div>
  );
}
