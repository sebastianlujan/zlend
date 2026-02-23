const logos: Record<string, { icon: React.ReactNode; color: string }> = {
  zcash: {
    color: "text-[#F4B728]",
    icon: (
      <svg width="32" height="32" viewBox="0 0 32 32" fill="none">
        <path
          d="M16 2L4 8v16l12 6 12-6V8L16 2z"
          stroke="#F4B728"
          strokeWidth="1.5"
          fill="none"
        />
        <text
          x="16"
          y="20"
          textAnchor="middle"
          fill="#F4B728"
          fontSize="14"
          fontWeight="bold"
          fontFamily="monospace"
        >
          Z
        </text>
      </svg>
    ),
  },
  avalanche: {
    color: "text-primary-500",
    icon: (
      <svg width="32" height="32" viewBox="0 0 32 32" fill="none">
        <path
          d="M16 4L3 28h26L16 4z"
          stroke="#E84142"
          strokeWidth="1.5"
          fill="none"
        />
        <text
          x="16"
          y="24"
          textAnchor="middle"
          fill="#E84142"
          fontSize="12"
          fontWeight="bold"
          fontFamily="monospace"
        >
          A
        </text>
      </svg>
    ),
  },
  ogbank: {
    color: "text-white",
    icon: (
      <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
        <rect x="3" y="11" width="18" height="11" rx="2" />
        <path d="M7 11V7a5 5 0 0110 0v4" />
      </svg>
    ),
  },
};

interface ChainNodeProps {
  chain: "zcash" | "avalanche" | "ogbank";
  label: string;
  sublabel?: string;
}

export function ChainNode({ chain, label, sublabel }: ChainNodeProps) {
  const { icon, color } = logos[chain];
  const borderGlow =
    chain === "zcash"
      ? "border-[#F4B728]/30 hover:border-[#F4B728]/60 hover:shadow-[0_0_20px_rgba(244,183,40,0.2)]"
      : chain === "avalanche"
        ? "border-primary-500/30 hover:border-primary-500/60 hover:shadow-[0_0_20px_rgba(232,65,66,0.2)]"
        : "border-surface-600/50 hover:border-surface-500/70 hover:shadow-[0_0_20px_rgba(255,255,255,0.08)]";

  return (
    <div className="flex flex-col items-center text-center" data-animate>
      <div
        className={`w-[72px] h-[72px] rounded-full border flex items-center justify-center transition-all duration-400 ${borderGlow}`}
      >
        {icon}
      </div>
      <span className={`mt-3 text-sm font-bold ${color}`}>{label}</span>
      {sublabel && (
        <span className="text-[10px] font-mono text-surface-500 uppercase tracking-widest mt-0.5">
          {sublabel}
        </span>
      )}
    </div>
  );
}
