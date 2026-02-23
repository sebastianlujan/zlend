interface LayerDetailProps {
  detail: string;
  terminalLine: string;
  expanded: boolean;
}

export function LayerDetail({ detail, terminalLine, expanded }: LayerDetailProps) {
  return (
    <div
      className={`overflow-hidden transition-all duration-400 ${expanded ? "max-h-60 opacity-100 mt-4" : "max-h-0 opacity-0"}`}
    >
      <div className="rounded-lg border border-surface-700/50 bg-surface-950/80 overflow-hidden">
        {/* Terminal chrome */}
        <div className="flex items-center gap-1.5 px-3 py-2 border-b border-surface-700/30">
          <span className="w-2.5 h-2.5 rounded-full bg-primary-500/60" />
          <span className="w-2.5 h-2.5 rounded-full bg-[#F4B728]/60" />
          <span className="w-2.5 h-2.5 rounded-full bg-accent-500/60" />
          <span className="ml-2 text-[10px] font-mono text-surface-600">terminal</span>
        </div>

        {/* Content */}
        <div className="p-3 space-y-2">
          <p className="text-xs text-surface-400 font-mono leading-relaxed">
            {detail}
          </p>
          <div className="flex items-center gap-2">
            <span className="text-[10px] text-accent-500 font-mono">$</span>
            <code className="text-[11px] text-surface-300 font-mono" data-terminal-line>
              {terminalLine}
            </code>
          </div>
        </div>
      </div>
    </div>
  );
}
