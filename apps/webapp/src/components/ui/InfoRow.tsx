interface InfoRowProps {
  label: string;
  value: React.ReactNode;
  bold?: boolean;
}

export function InfoRow({ label, value, bold = false }: InfoRowProps) {
  return (
    <div className="flex items-center justify-between py-1">
      <span className="text-sm text-surface-400">{label}</span>
      <span
        className={`font-mono text-sm ${bold ? "font-bold text-white" : "text-surface-200"}`}
      >
        {value}
      </span>
    </div>
  );
}
