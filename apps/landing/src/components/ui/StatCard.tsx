interface StatCardProps {
  value: string;
  unit?: string;
  label: string;
  subtext?: string;
}

export function StatCard({ value, unit, label, subtext }: StatCardProps) {
  return (
    <div className="text-center p-6" data-animate>
      <div className="text-4xl md:text-5xl font-bold text-white font-mono">
        {value}
        {unit && <span className="text-primary-400 ml-1 text-2xl md:text-3xl">{unit}</span>}
      </div>
      <div className="mt-2 text-surface-200 font-medium">{label}</div>
      {subtext && (
        <div className="mt-1 text-sm text-surface-400">{subtext}</div>
      )}
    </div>
  );
}
