import { useState } from "react";
import type { Game } from "../lib/types";
import StatusBadge from "./StatusBadge";
import StoreBadge from "./StoreBadge";
import ErrorBanner from "./ErrorBanner";
import ConfirmDialog from "./ConfirmDialog";

function formatPlayTime(secs: number) {
  if (secs === 0) return "Never played";
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  return h > 0 ? `${h}h ${m}m` : `${m}m`;
}

function Field({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div className="flex flex-col gap-0.5">
      <span className="text-zinc-500 uppercase text-[10px] tracking-wider">{label}</span>
      <span
        className={`text-zinc-200 break-all leading-snug ${mono ? "font-mono text-[10px]" : "text-xs"}`}
      >
        {value || "—"}
      </span>
    </div>
  );
}

interface Props {
  game: Game;
  isConfiguring: boolean;
  isLaunching: boolean;
  onClose: () => void;
  onConfigure: (id: string, clean: boolean) => Promise<void>;
  onLaunch: (id: string) => Promise<void>;
  onStop: (id: string) => Promise<void>;
  onDelete: (id: string) => void;
}

export default function GameDetail({
  game,
  isConfiguring,
  isLaunching,
  onClose,
  onConfigure,
  onLaunch,
  onStop,
  onDelete,
}: Props) {
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [configStep, setConfigStep] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  const canConfigure = game.status === "Detected";
  const canLaunch = game.status === "Configured" || game.status === "Ready";
  const needsClean = game.status === "Configured" || game.status === "Ready";
  const busy = isConfiguring || isLaunching;

  const handleConfigure = async (clean: boolean) => {
    setActionError(null);
    setConfigStep("Starting…");
    try {
      await onConfigure(game.id, clean);
      setConfigStep(null);
    } catch (e) {
      setConfigStep(null);
      setActionError(String(e));
    }
  };

  const handleLaunch = async () => {
    setActionError(null);
    try {
      await onLaunch(game.id);
    } catch (e) {
      setActionError(String(e));
    }
  };

  return (
    <>
      <aside className="w-72 shrink-0 border-l border-zinc-800 bg-zinc-900 overflow-y-auto flex flex-col">
        {/* Header */}
        <div className="flex items-start justify-between p-5 pb-3 border-b border-zinc-800">
          <h2 className="font-bold text-sm leading-tight pr-2">{game.title}</h2>
          {!isLaunching && (
            <button
              onClick={onClose}
              className="text-zinc-500 hover:text-white shrink-0 text-xs"
              aria-label="Close"
            >
              ✕
            </button>
          )}
        </div>

        {/* Badges */}
        <div className="flex gap-1.5 flex-wrap px-5 py-3 border-b border-zinc-800">
          <StoreBadge store={game.store_origin} />
          <StatusBadge status={game.status} />
          {game.crack_type && (
            <span className="text-xs px-1.5 py-0.5 rounded bg-zinc-700 text-zinc-300">
              {game.crack_type}
            </span>
          )}
        </div>

        {/* In-progress indicator */}
        {isConfiguring && (
          <div className="px-5 pt-3 pb-1">
            <div className="text-xs text-sky-400 animate-pulse">
              {configStep ?? "Configuring…"}
            </div>
            <div className="mt-1.5 h-1 bg-zinc-700 rounded-full overflow-hidden">
              <div className="h-full bg-sky-500 rounded-full animate-pulse w-3/4" />
            </div>
          </div>
        )}

        {isLaunching && (
          <div className="px-5 pt-3 pb-1">
            <div className="text-xs text-green-400 animate-pulse">Game running…</div>
            <div className="mt-1.5 h-1 bg-zinc-700 rounded-full overflow-hidden">
              <div className="h-full bg-green-500 rounded-full animate-pulse w-full" />
            </div>
          </div>
        )}

        {/* Fields */}
        <div className="flex flex-col gap-3 p-5">
          <Field label="Play time" value={formatPlayTime(game.play_time)} />
          <Field
            label="Last played"
            value={game.last_played ? game.last_played.slice(0, 10) : "Never"}
          />
          <Field label="Install dir" value={game.install_dir} mono />
          <Field label="Prefix" value={game.prefix_path} mono />
          <Field label="Runtime" value={game.runtime_id ?? ""} mono />
          <Field label="Added" value={game.added_at.slice(0, 10)} />
        </div>

        {/* Error states */}
        {game.status === "Broken" && (
          <div className="px-5 pb-3">
            <ErrorBanner message="Game is broken — reconfigure or verify install dir." />
          </div>
        )}
        {actionError && (
          <div className="px-5 pb-3">
            <ErrorBanner message={actionError} onDismiss={() => setActionError(null)} />
          </div>
        )}

        {/* Actions */}
        <div className="mt-auto flex flex-col gap-2 p-5 pt-3 border-t border-zinc-800">
          {isLaunching ? (
            <button
              onClick={() => onStop(game.id)}
              className="text-xs px-3 py-2 rounded text-white w-full transition-colors bg-red-700 hover:bg-red-600"
            >
              Stop
            </button>
          ) : (
            canLaunch && (
              <button
                onClick={handleLaunch}
                disabled={busy}
                className={`text-xs px-3 py-2 rounded text-white w-full transition-colors ${
                  busy
                    ? "bg-green-900 cursor-not-allowed opacity-50"
                    : "bg-green-700 hover:bg-green-600"
                }`}
              >
                Launch
              </button>
            )
          )}

          {canConfigure && (
            <button
              onClick={() => handleConfigure(false)}
              disabled={busy}
              className={`text-xs px-3 py-2 rounded text-white w-full transition-colors ${
                isConfiguring
                  ? "bg-sky-900 cursor-not-allowed opacity-70"
                  : busy
                  ? "bg-sky-900 cursor-not-allowed opacity-50"
                  : "bg-sky-700 hover:bg-sky-600"
              }`}
            >
              {isConfiguring ? "Configuring…" : "Configure"}
            </button>
          )}

          {needsClean && !isConfiguring && !isLaunching && (
            <button
              onClick={() => handleConfigure(true)}
              disabled={busy}
              className="text-xs px-3 py-2 rounded border border-zinc-600 text-zinc-300 hover:bg-zinc-800 w-full disabled:opacity-40"
            >
              Configure + Clean
            </button>
          )}

          {!busy && (
            <button
              onClick={() => setConfirmDelete(true)}
              className="text-xs px-3 py-2 rounded border border-red-900 text-red-400 hover:bg-red-950 w-full"
            >
              Delete
            </button>
          )}
        </div>
      </aside>

      {confirmDelete && (
        <ConfirmDialog
          title="Delete game"
          message={`Remove "${game.title}" from the library? Game files won't be deleted.`}
          confirmLabel="Delete"
          destructive
          onConfirm={() => {
            setConfirmDelete(false);
            onDelete(game.id);
          }}
          onCancel={() => setConfirmDelete(false)}
        />
      )}
    </>
  );
}
