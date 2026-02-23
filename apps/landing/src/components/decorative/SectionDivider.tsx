export function SectionDivider() {
  return (
    <div className="relative flex items-center justify-center py-4" aria-hidden="true">
      <div className="w-full max-w-6xl mx-auto px-6">
        <div className="relative h-px bg-surface-800/50">
          <div className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2">
            <div className="w-1.5 h-1.5 rounded-full bg-primary-500/60" />
          </div>
        </div>
      </div>
    </div>
  );
}
