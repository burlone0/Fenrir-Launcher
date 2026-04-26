interface Props {
  value: number;
  max: number;
  label?: string;
  className?: string;
}

export default function ProgressBar({ value, max, label, className = "" }: Props) {
  const pct = max > 0 ? Math.min(100, Math.round((value / max) * 100)) : 0;

  return (
    <div className={`flex flex-col gap-1 ${className}`}>
      {label && (
        <div className="flex justify-between text-xs text-zinc-400">
          <span className="truncate">{label}</span>
          <span className="shrink-0 ml-2">{pct}%</span>
        </div>
      )}
      <div className="h-1.5 bg-zinc-700 rounded-full overflow-hidden">
        <div
          className="h-full bg-sky-500 rounded-full transition-all duration-150"
          style={{ width: `${pct}%` }}
        />
      </div>
    </div>
  );
}
