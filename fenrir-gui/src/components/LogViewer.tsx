import { useEffect, useRef, useState } from "react";
import { readGameLog } from "../lib/commands";

interface Props {
  gameId: string;
  onOpenFolder: () => void;
}

export default function LogViewer({ gameId, onOpenFolder }: Props) {
  const [log, setLog] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const preRef = useRef<HTMLPreElement>(null);

  const load = async () => {
    setLoading(true);
    try {
      const text = await readGameLog(gameId);
      setLog(text);
    } catch {
      setLog(null);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
  }, [gameId]);

  useEffect(() => {
    if (preRef.current) {
      preRef.current.scrollTop = preRef.current.scrollHeight;
    }
  }, [log]);

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center justify-between">
        <span className="text-zinc-500 uppercase text-[10px] tracking-wider">Log output</span>
        <button
          onClick={load}
          disabled={loading}
          className="text-[10px] text-zinc-400 hover:text-white disabled:opacity-40 transition-colors"
        >
          Refresh
        </button>
      </div>

      {loading ? (
        <p className="text-zinc-500 text-xs italic">Loading…</p>
      ) : log !== null ? (
        <pre
          ref={preRef}
          className="bg-zinc-950 text-zinc-300 font-mono text-[10px] leading-relaxed p-3 rounded max-h-48 overflow-y-auto whitespace-pre-wrap break-all"
        >
          {log}
        </pre>
      ) : (
        <p className="text-zinc-500 text-xs italic">Play this game to generate log output</p>
      )}

      <button
        onClick={onOpenFolder}
        className="text-[10px] text-zinc-400 hover:text-white self-start transition-colors"
      >
        Open log folder
      </button>
    </div>
  );
}
