import type { Content } from "../../../data/content";

type World = Content["solution"]["worlds"][keyof Content["solution"]["worlds"]];

interface WorldPanelProps {
  world: World;
  side: "zcash" | "avalanche";
}

export function WorldPanel({ world, side }: WorldPanelProps) {
  const isZcash = side === "zcash";

  const borderIdle = isZcash ? "border-[#F4B728]/20" : "border-primary-500/20";
  const borderHover = isZcash ? "hover:border-[#F4B728]/40" : "hover:border-primary-500/40";
  const accentColor = isZcash ? "text-[#F4B728]" : "text-primary-400";
  const bgGlow = isZcash
    ? "bg-[radial-gradient(ellipse_at_center,rgba(244,183,40,0.06),transparent_70%)]"
    : "bg-[radial-gradient(ellipse_at_center,rgba(232,65,66,0.06),transparent_70%)]";

  const terminalComment = isZcash ? "// zcash_network" : "// avalanche_network";

  return (
    <div
      className={`relative rounded-xl border ${borderIdle} ${borderHover} ${bgGlow} h-[200px] md:h-[240px] overflow-hidden transition-colors duration-200`}
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

      {/* Content — spread top-to-bottom like FeatureBox */}
      <div className="relative z-10 p-6 md:p-8 h-full flex flex-col justify-between">
        {/* Top: terminal tag */}
        <span className={`text-xs font-mono ${accentColor} opacity-60 uppercase tracking-wider`}>
          {terminalComment}
        </span>

        {/* Bottom: icon + large label */}
        <div>
          <div
            className={`w-14 h-14 rounded-full border ${borderIdle} flex items-center justify-center mb-3`}
          >
            {isZcash ? (
              <svg width="28" height="28" viewBox="0 0 32 32" fill="none">
                <path d="M16 2L4 8v16l12 6 12-6V8L16 2z" stroke="#F4B728" strokeWidth="1.5" fill="none" />
                <text x="16" y="20" textAnchor="middle" fill="#F4B728" fontSize="14" fontWeight="bold" fontFamily="monospace">Z</text>
              </svg>
            ) : (
              <svg width="28" height="28" viewBox="0 0 32 32" fill="none">
                <path d="M16 4L3 28h26L16 4z" stroke="#E84142" strokeWidth="1.5" fill="none" />
                <text x="16" y="24" textAnchor="middle" fill="#E84142" fontSize="12" fontWeight="bold" fontFamily="monospace">A</text>
              </svg>
            )}
          </div>

          <h3 className={`text-2xl md:text-3xl font-bold font-mono leading-tight ${accentColor}`}>
            {world.label}
          </h3>
          <span className="block mt-1 text-xs font-mono text-surface-500 uppercase tracking-widest">
            {world.chain}
          </span>
        </div>
      </div>
    </div>
  );
}
