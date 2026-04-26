import { useState } from "react";
import { useGamesStore } from "../stores/games";
import { useUIStore } from "../stores/ui";
import GameGrid from "../components/GameGrid";
import GameDetail from "../components/GameDetail";
import ScanView from "./ScanView";
import type { GameStatus } from "../lib/types";

const STATUS_FILTERS: Array<GameStatus | "All"> = [
  "All",
  "Detected",
  "Configured",
  "Ready",
  "Broken",
];

export default function Library() {
  const {
    games,
    selectedId,
    isLoading,
    configuringId,
    launchingId,
    selectGame,
    configureGame,
    launchGame,
    deleteGame,
  } = useGamesStore();
  const { isScanOpen, openScan, notify } = useUIStore();

  const [filter, setFilter] = useState<GameStatus | "All">("All");

  const visible =
    filter === "All" ? games : games.filter((g) => g.status === filter);

  const selectedGame = games.find((g) => g.id === selectedId) ?? null;

  const handleConfigure = async (id: string, clean: boolean) => {
    try {
      await configureGame(id, clean);
      notify(`Configured: ${games.find((g) => g.id === id)?.title ?? id}`, "success");
    } catch (e) {
      notify(String(e), "error");
      throw e;
    }
  };

  const handleLaunch = async (id: string) => {
    try {
      await launchGame(id);
    } catch (e) {
      notify(String(e), "error");
      throw e;
    }
  };

  const handleDelete = async (id: string) => {
    const title = games.find((g) => g.id === id)?.title ?? id;
    await deleteGame(id);
    notify(`Deleted: ${title}`, "info");
  };

  return (
    <div className="flex h-full overflow-hidden">
      {/* Main area */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Toolbar */}
        <div className="flex items-center gap-2 px-6 py-3 border-b border-zinc-800 shrink-0">
          <span className="text-zinc-500 text-xs">{games.length} games</span>

          <div className="flex gap-1 ml-auto">
            {STATUS_FILTERS.map((s) => (
              <button
                key={s}
                onClick={() => setFilter(s)}
                className={`text-xs px-2 py-1 rounded transition-colors ${
                  filter === s ? "bg-zinc-700 text-white" : "text-zinc-500 hover:text-zinc-200"
                }`}
              >
                {s}
              </button>
            ))}
          </div>

          <button
            onClick={openScan}
            className="text-xs px-3 py-1.5 rounded bg-sky-700 hover:bg-sky-600 text-white ml-2 shrink-0"
          >
            Scan
          </button>
        </div>

        {/* Grid */}
        <div className="flex-1 overflow-y-auto p-6">
          {isLoading ? (
            <div className="text-zinc-500 text-sm">Loading library…</div>
          ) : (
            <GameGrid
              games={visible}
              selectedId={selectedId}
              configuringId={configuringId}
              launchingId={launchingId}
              onSelect={selectGame}
              onConfigure={(id) => {
                const g = games.find((x) => x.id === id);
                if (g?.status === "Configured" || g?.status === "Ready") {
                  handleLaunch(id);
                } else {
                  handleConfigure(id, false);
                }
              }}
              onLaunch={handleLaunch}
            />
          )}
        </div>
      </div>

      {/* Detail panel */}
      {selectedGame && (
        <GameDetail
          game={selectedGame}
          isConfiguring={configuringId === selectedGame.id}
          isLaunching={launchingId === selectedGame.id}
          onClose={() => selectGame(null)}
          onConfigure={handleConfigure}
          onLaunch={handleLaunch}
          onDelete={handleDelete}
        />
      )}

      {/* Scan overlay */}
      {isScanOpen && <ScanView />}
    </div>
  );
}
