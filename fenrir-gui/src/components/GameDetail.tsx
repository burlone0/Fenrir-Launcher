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
      <span className={`text-zinc-200 break-all leading-snug ${mono ? "font-mono text-[10px]" : "text-xs"}`}>
        {value || "—"}
      </span>
    </div>
  );
}

interface Props {
  game: Game;
  onClose: () => void;
  onConfigure: (id: string, clean: boolean) => void;
  onLaunch: (id: string) => void;
  onDelete: (id: string) => void;
}

export default function GameDetail({ game, onClose, onConfigure, onLaunch, onDelete }: Props) {
  const [confirmDelete, setConfirmDelete] = useState(false);

  const canConfigure = game.status === "Detected" || game.status === "Configured";
  const canLaunch = game.status === "Ready";
  const needsClean = game.status === "Configured";

  return (
    <>
      <aside className="w-72 shrink-0 border-l border-zinc-800 bg-zinc-900 overflow-y-auto flex flex-col">
        {/* Header */}
        <div className="flex items-start justify-between p-5 pb-3 border-b border-zinc-800">
          <h2 className="font-bold text-sm leading-tight pr-2">{game.title}</h2>
          <button
            onClick={onClose}
            className="text-zinc-500 hover:text-white shrink-0 text-xs"
            aria-label="Close"
          >
            ✕
          </button>
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

        {/* Error state */}
        {game.status === "Broken" && (
          <div className="px-5 pb-3">
            <ErrorBanner message="Game is broken — reconfigure or verify install dir." />
          </div>
        )}

        {/* Actions */}
        <div className="mt-auto flex flex-col gap-2 p-5 pt-3 border-t border-zinc-800">
          {canLaunch && (
            <button
              onClick={() => onLaunch(game.id)}
              className="text-xs px-3 py-2 rounded bg-green-700 hover:bg-green-600 text-white w-full"
            >
              Launch
            </button>
          )}
          {canConfigure && (
            <button
              onClick={() => onConfigure(game.id, false)}
              className="text-xs px-3 py-2 rounded bg-sky-700 hover:bg-sky-600 text-white w-full"
            >
              Configure
            </button>
          )}
          {needsClean && (
            <button
              onClick={() => onConfigure(game.id, true)}
              className="text-xs px-3 py-2 rounded border border-zinc-600 text-zinc-300 hover:bg-zinc-800 w-full"
            >
              Configure + Clean
            </button>
          )}
          <button
            onClick={() => setConfirmDelete(true)}
            className="text-xs px-3 py-2 rounded border border-red-900 text-red-400 hover:bg-red-950 w-full"
          >
            Delete
          </button>
        </div>
      </aside>

      {confirmDelete && (
        <ConfirmDialog
          title="Delete game"
          message={`Remove "${game.title}" from the library? The game files won't be deleted.`}
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
