import type { Content } from "../../../data/content";

type Bridge = Content["solution"]["bridge"];

interface BridgeColumnProps {
  bridge: Bridge;
}

export function BridgeColumn({ bridge }: BridgeColumnProps) {
  return (
    <div
      className="flex flex-col items-center text-center"
      data-bridge-column
    >
      <div className="w-14 h-14 rounded-full border border-surface-600/50 flex items-center justify-center mx-auto mb-2 hover:border-surface-500/70 transition-colors duration-200">
        <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
          <rect x="3" y="11" width="18" height="11" rx="2" />
          <path d="M7 11V7a5 5 0 0110 0v4" />
        </svg>
      </div>
      <span className="text-sm font-bold text-white">{bridge.label}</span>
      <span className="block text-[10px] font-mono text-surface-500 mt-0.5">
        {bridge.subtitle}
      </span>
    </div>
  );
}
