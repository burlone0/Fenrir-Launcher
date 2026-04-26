import { useState } from "react";
import { useGamesStore } from "../stores/games";
import { useUIStore } from "../stores/ui";
import type { Game } from "../lib/types";

function statusColor(status: Game["status"]) {
  switch (status) {
    case "Ready": return "bg-green-700 text-green-100";
    case "Configured": return "bg-blue-700 text-blue-100";
    case "Detected": return "bg-zinc-600 text-zinc-200";
    case "Broken": return "bg-red-700 text-red-100";
    case "NeedsConfirmation": return "bg-yellow-700 text-yellow-100";
  }
}

function storeColor(store: Game["store_origin"]) {
  switch (store) {
    case "Steam": return "bg-sky-800 text-sky-100";
    case "GOG": return "bg-purple-800 text-purple-100";
    case "Epic": return "bg-orange-800 text-orange-100";
    default: return "bg-zinc-700 text-zinc-200";
  }
}

function formatPlayTime(secs: number) {
  if (secs === 0) return "—";
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  return h > 0 ? `${h}h ${m}m` : `${m}m`;
}

function GameCard({ game }: { game: Game }) {
  const { selectedId, selectGame, configureGame, launchGame } = useGamesStore();
  const { notify } = useUIStore();
  const selected = selectedId === game.id;

  const handleAction = async (e: React.MouseEvent) => {
    e.stopPropagation();
    if (game.status === "Detected" || game.status === "Configured") {
      await configureGame(game.id, false);
      notify(`Configured: ${game.title}`, "success");
    } else if (game.status === "Ready") {
      await launchGame(game.id);
      notify(`Launched: ${game.title}`, "info");
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
      onClick={() => selectGame(selected ? null : game.id)}
      className={`rounded-lg border p-4 cursor-pointer transition-colors flex flex-col gap-2 ${
        selected
          ? "border-sky-500 bg-zinc-800"
          : "border-zinc-700 bg-zinc-900 hover:border-zinc-500"
      }`}
    >
      <div className="font-semibold text-sm truncate">{game.title}</div>
      <div className="flex gap-1 flex-wrap">
        <span className={`text-xs px-1.5 py-0.5 rounded ${storeColor(game.store_origin)}`}>
          {game.store_origin}
        </span>
        <span className={`text-xs px-1.5 py-0.5 rounded ${statusColor(game.status)}`}>
          {game.status}
        </span>
        {game.crack_type && (
          <span className="text-xs px-1.5 py-0.5 rounded bg-zinc-700 text-zinc-300">
            {game.crack_type}
          </span>
        )}
      </div>
      <div className="text-xs text-zinc-500">{formatPlayTime(game.play_time)}</div>
      {actionLabel && (
        <button
          onClick={handleAction}
          className="mt-1 text-xs px-3 py-1 rounded bg-sky-700 hover:bg-sky-600 text-white self-start"
        >
          {actionLabel}
        </button>
      )}
    </div>
  );
}

function GameDetail() {
  const { games, selectedId, selectGame, deleteGame } = useGamesStore();
  const { notify } = useUIStore();
  const game = games.find((g) => g.id === selectedId);
  if (!game) return null;

  const handleDelete = async () => {
    await deleteGame(game.id);
    notify(`Deleted: ${game.title}`, "info");
  };

  return (
    <aside className="w-72 shrink-0 border-l border-zinc-800 bg-zinc-900 overflow-y-auto p-5 flex flex-col gap-4">
      <div className="flex items-start justify-between">
        <h2 className="font-bold text-base leading-tight">{game.title}</h2>
        <button
          onClick={() => selectGame(null)}
          className="text-zinc-500 hover:text-white text-sm"
        >
          ✕
        </button>
      </div>

      <div className="flex flex-col gap-1 text-xs">
        <Row label="Store" value={game.store_origin} />
        <Row label="Crack" value={game.crack_type ?? "—"} />
        <Row label="Status" value={game.status} />
        <Row label="Play time" value={formatPlayTime(game.play_time)} />
        <Row label="Last played" value={game.last_played?.slice(0, 10) ?? "Never"} />
        <Row label="Install dir" value={game.install_dir} mono />
        <Row label="Prefix" value={game.prefix_path || "—"} mono />
        <Row label="Runtime" value={game.runtime_id ?? "—"} mono />
      </div>

      {game.status === "Broken" && (
        <div className="text-xs text-red-400 bg-red-950 border border-red-800 rounded p-2">
          Game is broken — reconfigure or check install dir.
        </div>
      )}

      <div className="flex flex-col gap-2 mt-auto pt-4 border-t border-zinc-800">
        <button
          onClick={() => handleDelete()}
          className="text-xs px-3 py-1.5 rounded border border-red-800 text-red-400 hover:bg-red-950"
        >
          Delete
        </button>
      </div>
    </aside>
  );
}

function Row({
  label,
  value,
  mono,
}: {
  label: string;
  value: string;
  mono?: boolean;
}) {
  return (
    <div className="flex flex-col gap-0.5">
      <span className="text-zinc-500 uppercase text-[10px] tracking-wider">{label}</span>
      <span className={`text-zinc-200 break-all ${mono ? "font-mono text-[10px]" : ""}`}>
        {value}
      </span>
    </div>
  );
}

export default function Library() {
  const { games, isLoading } = useGamesStore();
  const { openScan, isScanOpen } = useUIStore();

  const statusFilter = ["All", "Detected", "Configured", "Ready", "Broken"];
  const [filter, setFilter] = useState("All");

  const visible =
    filter === "All" ? games : games.filter((g) => g.status === filter);

  return (
    <div className="flex h-full">
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Header */}
        <div className="flex items-center gap-3 px-6 py-4 border-b border-zinc-800">
          <span className="text-zinc-400 text-sm">{games.length} games</span>
          <div className="flex gap-1 ml-auto">
            {statusFilter.map((s) => (
              <button
                key={s}
                onClick={() => setFilter(s)}
                className={`text-xs px-2 py-1 rounded ${
                  filter === s
                    ? "bg-zinc-700 text-white"
                    : "text-zinc-500 hover:text-white"
                }`}
              >
                {s}
              </button>
            ))}
          </div>
          <button
            onClick={openScan}
            className="text-xs px-3 py-1.5 rounded bg-sky-700 hover:bg-sky-600 text-white ml-2"
          >
            Scan
          </button>
        </div>

        {/* Grid */}
        <div className="flex-1 overflow-y-auto p-6">
          {isLoading ? (
            <div className="text-zinc-500 text-sm">Loading...</div>
          ) : visible.length === 0 ? (
            <div className="text-zinc-500 text-sm">
              No games found. Run Scan to detect games.
            </div>
          ) : (
            <div className="grid grid-cols-2 xl:grid-cols-3 gap-3">
              {visible.map((g) => (
                <GameCard key={g.id} game={g} />
              ))}
            </div>
          )}
        </div>
      </div>

      <GameDetail />

      {isScanOpen && (
        <div className="fixed inset-0 bg-black/60 z-40 flex items-center justify-center">
          <div className="bg-zinc-900 border border-zinc-700 rounded-lg p-6 w-[480px] text-sm">
            <div className="font-semibold mb-4">Scan for games</div>
            <div className="text-zinc-400 text-xs">
              Sprint 3 — ScanView component coming soon.
            </div>
            <button
              onClick={() => useUIStore.getState().closeScan()}
              className="mt-4 text-xs px-3 py-1.5 rounded bg-zinc-700 hover:bg-zinc-600"
            >
              Close
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
