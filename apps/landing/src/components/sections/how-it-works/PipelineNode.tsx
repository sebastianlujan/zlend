import type { Content } from "../../../data/content";

type PipelineStep = Content["howItWorks"]["pipeline"][number];

const icons: Record<string, React.ReactNode> = {
  deposit: (
    <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <path d="M12 2L3 7v10l9 5 9-5V7l-9-5z" />
      <path d="M12 22V12" />
      <path d="M12 12L3 7" />
      <path d="M12 12l9-5" />
    </svg>
  ),
  proof: (
    <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
      <path d="M7 11V7a5 5 0 0110 0v4" />
      <circle cx="12" cy="16" r="1" />
    </svg>
  ),
  verify: (
    <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
      <path d="M9 12l2 2 4-4" />
    </svg>
  ),
  borrow: (
    <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="10" />
      <path d="M16 8h-6a2 2 0 100 4h4a2 2 0 110 4H8" />
      <path d="M12 6v2" />
      <path d="M12 16v2" />
    </svg>
  ),
};

export function PipelineNode({ step }: { step: PipelineStep }) {
  const colorClass = step.color === "primary" ? "text-primary-500" : "text-accent-500";

  return (
    <div
      className="flex flex-col items-center text-center w-[140px] lg:w-[150px]"
      data-pipeline-node
    >
      <div className="pipeline-icon-ring" data-color={step.color}>
        <span className={colorClass}>{icons[step.id]}</span>
      </div>

      <span className={`mt-3 text-[11px] font-mono tracking-wider ${colorClass}`}>
        {step.step}
      </span>

      <h3 className="mt-1 text-base font-semibold text-white leading-tight">
        {step.label}
      </h3>

      <span className="mt-1 text-[9px] font-mono text-surface-600 uppercase tracking-[0.15em]">
        {step.network}
      </span>

      <p className="mt-2 text-xs text-surface-400 font-mono leading-relaxed">
        {step.description}
      </p>
    </div>
  );
}
