import type { Content } from "../../../data/content";

type World = Content["solution"]["worlds"][keyof Content["solution"]["worlds"]];

interface WorldPanelProps {
  world: World;
  side: "zcash" | "avalanche";
}

export function WorldPanel({ world, side }: WorldPanelProps) {
  const isZcash = side === "zcash";

  const borderColor = isZcash
    ? "border-[#F4B728]/20"
    : "border-primary-500/20";
  const accentColor = isZcash ? "text-[#F4B728]" : "text-primary-400";
  const bgGlow = isZcash
    ? "bg-[radial-gradient(ellipse_at_center,rgba(244,183,40,0.06),transparent_70%)]"
    : "bg-[radial-gradient(ellipse_at_center,rgba(232,65,66,0.06),transparent_70%)]";

  return (
    <div
      className={`relative rounded-2xl border ${borderColor} p-6 md:p-8 ${bgGlow} overflow-hidden`}
      data-world-panel={side}
    >
      {/* Dot pattern overlay */}
      <div
        className="absolute inset-0 opacity-[0.03] pointer-events-none"
        style={{
          backgroundImage: `radial-gradient(circle 1px at center, currentColor 1px, transparent 1px)`,
          backgroundSize: "24px 24px",
        }}
      />

      <div className="relative z-10 flex items-center gap-3">
        <div
          className={`w-10 h-10 rounded-full border ${borderColor} flex items-center justify-center`}
        >
          {isZcash ? (
            <svg width="20" height="20" viewBox="0 0 32 32" fill="none">
              <path d="M16 2L4 8v16l12 6 12-6V8L16 2z" stroke="#F4B728" strokeWidth="1.5" fill="none" />
              <text x="16" y="20" textAnchor="middle" fill="#F4B728" fontSize="14" fontWeight="bold" fontFamily="monospace">Z</text>
            </svg>
          ) : (
            <svg width="20" height="20" viewBox="0 0 32 32" fill="none">
              <path d="M16 4L3 28h26L16 4z" stroke="#E84142" strokeWidth="1.5" fill="none" />
              <text x="16" y="24" textAnchor="middle" fill="#E84142" fontSize="12" fontWeight="bold" fontFamily="monospace">A</text>
            </svg>
          )}
        </div>
        <div>
          <span className={`text-sm font-bold ${accentColor}`}>
            {world.label}
          </span>
          <span className="block text-[10px] font-mono text-surface-500 uppercase tracking-widest">
            {world.chain}
          </span>
        </div>
      </div>
    </div>
  );
}
