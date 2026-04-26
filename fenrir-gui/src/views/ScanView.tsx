import { useState } from "react";
import { useUIStore } from "../stores/ui";
import { useScanStore } from "../stores/scan";
import { useGamesStore } from "../stores/games";
import ProgressBar from "../components/ProgressBar";
import StoreBadge from "../components/StoreBadge";
import type { ClassifiedGame } from "../lib/types";

type Phase = "input" | "scanning" | "results";

function ClassifiedRow({
  game,
  confirmed,
  onConfirm,
}: {
  game: ClassifiedGame;
  confirmed: boolean;
  onConfirm: () => void;
}) {
  return (
    <div className="flex items-center gap-3 py-2 border-t border-zinc-800">
      <div className="flex-1 min-w-0">
        <div className="text-sm font-medium truncate">{game.title}</div>
        <div className="flex gap-1 mt-0.5">
          <StoreBadge store={game.store_origin} />
          {game.crack_type && (
            <span className="text-xs px-1.5 py-0.5 rounded bg-zinc-700 text-zinc-300">
              {game.crack_type}
            </span>
          )}
          <span className="text-xs text-zinc-500 ml-1">{game.confidence}pts</span>
        </div>
      </div>
      {confirmed ? (
        <span className="text-xs text-green-400 shrink-0">✓ Added</span>
      ) : (
        <button
          onClick={onConfirm}
          className="text-xs px-3 py-1 rounded bg-sky-700 hover:bg-sky-600 text-white shrink-0"
        >
          Confirm
        </button>
      )}
    </div>
  );
}

export default function ScanView() {
  const { closeScan } = useUIStore();
  const { isScanning, progress, lastResult, scanDirectory, confirmGame, clearResult } =
    useScanStore();
  const { loadGames } = useGamesStore();

  const [path, setPath] = useState("");
  const [phase, setPhase] = useState<Phase>("input");
  const [confirmed, setConfirmed] = useState<Set<string>>(new Set());

  const handleScan = async () => {
    if (!path.trim() && phase === "input") {
      // allow empty = use configured dirs
    }
    setPhase("scanning");
    await scanDirectory(path.trim() || undefined);
    // TODO Sprint 5: phase transition driven by scan:done event
    // For now, simulate completion with mock
    setPhase("results");
  };

  const handleConfirm = async (game: ClassifiedGame) => {
    await confirmGame(game.title);
    setConfirmed((prev) => new Set(prev).add(game.path));
  };

  const handleDone = async () => {
    await loadGames();
    clearResult();
    setPhase("input");
    setPath("");
    setConfirmed(new Set());
    closeScan();
  };

  return (
    <div className="fixed inset-0 bg-black/60 z-40 flex items-center justify-center">
      <div className="bg-zinc-900 border border-zinc-700 rounded-lg w-[540px] max-w-[95vw] max-h-[80vh] flex flex-col overflow-hidden">

        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-800">
          <h2 className="font-semibold">
            {phase === "input" && "Scan for games"}
            {phase === "scanning" && "Scanning…"}
            {phase === "results" && "Scan complete"}
          </h2>
          <button
            onClick={handleDone}
            disabled={isScanning}
            className="text-zinc-500 hover:text-white disabled:opacity-30 text-sm"
            aria-label="Close"
          >
            ✕
          </button>
        </div>

        {/* Body */}
        <div className="flex-1 overflow-y-auto px-6 py-5 flex flex-col gap-4">

          {/* Phase: input */}
          {phase === "input" && (
            <>
              <p className="text-sm text-zinc-400">
                Enter the folder containing your games. Leave empty to use
                directories configured in <code className="text-zinc-300">~/.config/fenrir/config.toml</code>.
              </p>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={path}
                  onChange={(e) => setPath(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && handleScan()}
                  placeholder="/mnt/games"
                  className="flex-1 bg-zinc-800 border border-zinc-700 rounded px-3 py-2 text-sm text-zinc-200 placeholder-zinc-600 focus:outline-none focus:border-sky-500"
                />
                <button
                  onClick={handleScan}
                  className="px-4 py-2 rounded bg-sky-700 hover:bg-sky-600 text-white text-sm shrink-0"
                >
                  Scan
                </button>
              </div>
            </>
          )}

          {/* Phase: scanning */}
          {phase === "scanning" && (
            <div className="flex flex-col gap-4">
              <ProgressBar
                value={progress?.current ?? 0}
                max={progress?.total ?? 1}
                label={progress?.path ?? "Initializing…"}
              />
              <p className="text-xs text-zinc-500 text-center">
                {progress
                  ? `${progress.current} / ${progress.total} candidates`
                  : "Starting scan…"}
              </p>
            </div>
          )}

          {/* Phase: results */}
          {phase === "results" && (
            <>
              {/* High confidence — auto-added */}
              {(lastResult?.high_confidence?.length ?? 0) > 0 && (
                <section>
                  <div className="flex items-center gap-2 mb-1">
                    <h3 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider">
                      Auto-added
                    </h3>
                    <span className="text-xs text-zinc-600">
                      ({lastResult!.high_confidence.length})
                    </span>
                  </div>
                  {lastResult!.high_confidence.map((g) => (
                    <div
                      key={g.path}
                      className="flex items-center gap-3 py-2 border-t border-zinc-800"
                    >
                      <div className="flex-1 min-w-0">
                        <div className="text-sm font-medium truncate">{g.title}</div>
                        <div className="flex gap-1 mt-0.5">
                          <StoreBadge store={g.store_origin} />
                          {g.crack_type && (
                            <span className="text-xs px-1.5 py-0.5 rounded bg-zinc-700 text-zinc-300">
                              {g.crack_type}
                            </span>
                          )}
                        </div>
                      </div>
                      <span className="text-xs text-green-400 shrink-0">✓ Added</span>
                    </div>
                  ))}
                </section>
              )}

              {/* Needs confirmation */}
              {(lastResult?.needs_confirmation?.length ?? 0) > 0 && (
                <section>
                  <div className="flex items-center gap-2 mb-1">
                    <h3 className="text-xs font-semibold text-zinc-400 uppercase tracking-wider">
                      Needs confirmation
                    </h3>
                    <span className="text-xs text-zinc-600">
                      ({lastResult!.needs_confirmation.length})
                    </span>
                  </div>
                  {lastResult!.needs_confirmation.map((g) => (
                    <ClassifiedRow
                      key={g.path}
                      game={g}
                      confirmed={confirmed.has(g.path)}
                      onConfirm={() => handleConfirm(g)}
                    />
                  ))}
                </section>
              )}

              {/* Empty result */}
              {(lastResult?.total ?? 0) === 0 && (
                <p className="text-sm text-zinc-500 text-center py-4">
                  No games found in the scanned directory.
                </p>
              )}

              {/* Mock data notice (removed in Sprint 4) */}
              {lastResult === null && (
                <p className="text-sm text-zinc-500 text-center py-4">
                  Scan complete. Results will appear here in Sprint 5 when
                  real scan events are wired.
                </p>
              )}
            </>
          )}
        </div>

        {/* Footer */}
        {phase === "results" && (
          <div className="flex justify-end px-6 py-4 border-t border-zinc-800">
            <button
              onClick={handleDone}
              className="text-sm px-4 py-2 rounded bg-zinc-700 hover:bg-zinc-600 text-white"
            >
              Done
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
