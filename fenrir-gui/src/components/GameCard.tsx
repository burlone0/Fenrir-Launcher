import type { Game } from "../lib/types";
import { coverFor } from "../lib/coverFor";
import StatusBadge from "./StatusBadge";

function formatPlayTime(secs: number) {
  if (secs === 0) return null;
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  return h > 0 ? `${h}h ${m}m` : `${m}m`;
}

interface Props {
  game: Game;
  selected: boolean;
  isConfiguring: boolean;
  isLaunching: boolean;
  onSelect: () => void;
  onConfigure: () => void;
  onLaunch: () => void;
}

export default function GameCard({
  game,
  selected,
  isConfiguring,
  isLaunching,
  onSelect,
  onConfigure,
  onLaunch,
}: Props) {
  const playTime = formatPlayTime(game.play_time);
  const busy = isConfiguring || isLaunching;
  const cover = coverFor(game.title);

  const handleAction = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (busy) return;
    if (game.status === "Detected" || game.status === "Configured") {
      onConfigure();
    } else if (game.status === "Ready") {
      onLaunch();
    }
  };

  const actionLabel = isConfiguring
    ? "Configuring…"
    : isLaunching
    ? "Running…"
    : game.status === "Detected"
    ? "Configure"
    : game.status === "Configured" || game.status === "Ready"
    ? "Launch"
    : null;

  const actionColor =
    isLaunching || game.status === "Configured" || game.status === "Ready"
      ? "bg-green-700 text-white"
      : "bg-sky-700 text-white";

  return (
    <div
      onClick={onSelect}
      className={`rounded-lg border overflow-hidden cursor-pointer transition-colors flex flex-col group ${
        selected
          ? "border-sky-500 bg-zinc-800"
          : "border-zinc-700 bg-zinc-900 hover:border-zinc-500"
      }`}
    >
      {/* Cover art — portrait 2:3 */}
      <div
        className="aspect-[2/3] w-full flex items-center justify-center relative overflow-hidden"
        style={{ backgroundColor: cover.bgColor }}
      >
        <div className="absolute inset-0 bg-black/25" />
        <span className="relative z-10 text-white font-bold text-4xl select-none opacity-70">
          {cover.initials}
        </span>

        {/* Status strip top-left */}
        <div className="absolute top-1.5 left-1.5 z-10">
          <StatusBadge status={game.status} />
        </div>

        {/* Broken warning */}
        {game.status === "Broken" && (
          <div className="absolute bottom-1.5 left-1.5 z-10 text-[10px] text-red-300 bg-black/60 rounded px-1">
            ⚠ broken
          </div>
        )}

        {/* Action button — visible on hover or when busy */}
        {actionLabel && (
          <button
            onClick={handleAction}
            disabled={busy}
            className={`absolute bottom-2 right-2 z-10 text-[10px] px-2 py-1 rounded font-medium transition-all ${actionColor} ${
              busy
                ? "opacity-80 cursor-not-allowed animate-pulse"
                : "opacity-0 group-hover:opacity-100"
            }`}
          >
            {actionLabel}
          </button>
        )}

        {/* Running indicator */}
        {isLaunching && (
          <div className="absolute inset-0 z-20 bg-black/40 flex items-center justify-center">
            <span className="text-green-400 text-[10px] font-medium animate-pulse">Running…</span>
          </div>
        )}
      </div>

      {/* Title + playtime */}
      <div className="px-2 py-1.5 flex flex-col gap-0.5">
        <div className="text-xs font-semibold truncate leading-snug" title={game.title}>
          {game.title}
        </div>
        {playTime && (
          <div className="text-[10px] text-zinc-500">{playTime}</div>
        )}
      </div>
    </div>
  );
}
