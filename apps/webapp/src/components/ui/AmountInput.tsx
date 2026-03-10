interface AmountInputProps {
  value: string;
  onChange: (value: string) => void;
  symbol: string;
  maxAmount?: string;
  disabled?: boolean;
  placeholder?: string;
}

export function AmountInput({
  value,
  onChange,
  symbol,
  maxAmount,
  disabled = false,
  placeholder = "0.00",
}: AmountInputProps) {
  const handleMax = () => {
    if (maxAmount) onChange(maxAmount);
  };

  return (
    <div className="flex items-center gap-2 rounded-lg border border-surface-700/50 bg-surface-800/50 px-4 py-3">
      <input
        type="text"
        inputMode="decimal"
        value={value}
        onChange={(e) => {
          const v = e.target.value;
          if (/^[0-9]*\.?[0-9]*$/.test(v)) onChange(v);
        }}
        placeholder={placeholder}
        disabled={disabled}
        className="flex-1 bg-transparent font-mono text-lg text-white placeholder:text-surface-500 focus:outline-none disabled:text-surface-500"
      />
      {maxAmount && (
        <button
          type="button"
          onClick={handleMax}
          disabled={disabled}
          className="rounded bg-surface-700 px-2 py-0.5 text-xs font-medium text-surface-300 transition-colors hover:bg-surface-600 cursor-pointer disabled:opacity-50"
        >
          MAX
        </button>
      )}
      <span className="font-mono text-sm text-surface-400">{symbol}</span>
    </div>
  );
}
