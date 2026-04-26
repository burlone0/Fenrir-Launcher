import type { Game } from "../lib/types";
import StatusBadge from "./StatusBadge";
import StoreBadge from "./StoreBadge";

function formatPlayTime(secs: number) {
  if (secs === 0) return null;
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  return h > 0 ? `${h}h ${m}m` : `${m}m`;
}

interface Props {
  game: Game;
  selected: boolean;
  onSelect: () => void;
  onConfigure: () => void;
  onLaunch: () => void;
}

export default function GameCard({ game, selected, onSelect, onConfigure, onLaunch }: Props) {
  const playTime = formatPlayTime(game.play_time);

  const handleAction = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (game.status === "Detected" || game.status === "Configured") {
      onConfigure();
    } else if (game.status === "Ready") {
      onLaunch();
    }
  };

  const actionLabel =
    game.status === "Detected" || game.status === "Configured"
      ? "Configure"
      : game.status === "Ready"
      ? "Launch"
      : null;

  return (
    <div
      onClick={onSelect}
      className={`rounded-lg border p-4 cursor-pointer transition-colors flex flex-col gap-2 ${
        selected
          ? "border-sky-500 bg-zinc-800"
          : "border-zinc-700 bg-zinc-900 hover:border-zinc-500 hover:bg-zinc-800/50"
      }`}
    >
      <div className="font-semibold text-sm truncate" title={game.title}>
        {game.title}
      </div>

      <div className="flex gap-1 flex-wrap">
        <StoreBadge store={game.store_origin} />
        <StatusBadge status={game.status} />
        {game.crack_type && (
          <span className="text-xs px-1.5 py-0.5 rounded bg-zinc-700 text-zinc-300">
            {game.crack_type}
          </span>
        )}
      </div>

      {playTime && (
        <div className="text-xs text-zinc-500">{playTime} played</div>
      )}

      {game.status === "Broken" && (
        <div className="text-xs text-red-400">⚠ needs attention</div>
      )}

      {actionLabel && (
        <button
          onClick={handleAction}
          className={`mt-auto text-xs px-3 py-1.5 rounded self-start transition-colors ${
            actionLabel === "Launch"
              ? "bg-green-700 hover:bg-green-600 text-white"
              : "bg-sky-700 hover:bg-sky-600 text-white"
          }`}
        >
          {actionLabel}
        </button>
      )}
    </div>
  );
}
