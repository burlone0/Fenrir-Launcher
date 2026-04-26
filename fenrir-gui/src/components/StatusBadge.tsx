import type { GameStatus } from "../lib/types";

const CONFIG: Record<GameStatus, { label: string; classes: string }> = {
  Ready:             { label: "Ready",              classes: "bg-green-800 text-green-100" },
  Configured:        { label: "Configured",         classes: "bg-blue-800 text-blue-100" },
  Detected:          { label: "Detected",           classes: "bg-zinc-600 text-zinc-200" },
  Broken:            { label: "Broken",             classes: "bg-red-800 text-red-100" },
  NeedsConfirmation: { label: "Needs confirmation", classes: "bg-yellow-700 text-yellow-100" },
};

export default function StatusBadge({ status }: { status: GameStatus }) {
  const { label, classes } = CONFIG[status];
  return (
    <span className={`inline-block text-xs px-1.5 py-0.5 rounded font-medium ${classes}`}>
      {label}
    </span>
  );
}
