import type { Content } from "../../../data/content";

type Bridge = Content["solution"]["bridge"];

interface BridgeColumnProps {
  bridge: Bridge;
}

export function BridgeColumn({ bridge }: BridgeColumnProps) {
  return (
    <div
      className="flex flex-col items-center justify-center text-center lg:h-[200px] md:h-[240px]"
      data-bridge-column
    >
      {/* Top connector line (desktop only) */}
      <div className="hidden lg:block w-px h-6 border-l border-dashed border-surface-600/50" />

      {/* Bridge card */}
      <div className="rounded-xl border border-surface-700/30 bg-surface-900/60 backdrop-blur-sm p-5 hover:border-surface-600/50 transition-colors duration-200">
        <span className="block text-[10px] font-mono text-surface-600 uppercase tracking-wider mb-3">
          {"// bridge_protocol"}
        </span>

        <div className="w-16 h-16 rounded-full border border-surface-600/40 flex items-center justify-center mx-auto mb-3">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
            <rect x="3" y="11" width="18" height="11" rx="2" />
            <path d="M7 11V7a5 5 0 0110 0v4" />
          </svg>
        </div>

        <span className="block text-lg md:text-xl font-bold text-white font-mono">
          {bridge.label}
        </span>
        <span className="block text-xs font-mono text-surface-500 mt-1 max-w-[160px] leading-relaxed mx-auto">
          {bridge.subtitle}
        </span>
      </div>

      {/* Bottom connector line (desktop only) */}
      <div className="hidden lg:block w-px h-6 border-l border-dashed border-surface-600/50" />
    </div>
  );
}
