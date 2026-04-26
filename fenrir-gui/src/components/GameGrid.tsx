import type { Game } from "../lib/types";
import GameCard from "./GameCard";

interface Props {
  games: Game[];
  selectedId: string | null;
  onSelect: (id: string | null) => void;
  onConfigure: (id: string) => void;
  onLaunch: (id: string) => void;
}

export default function GameGrid({ games, selectedId, onSelect, onConfigure, onLaunch }: Props) {
  if (games.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-3 text-zinc-500">
        <span className="text-4xl">🎮</span>
        <p className="text-sm">No games found. Run Scan to detect games.</p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4 gap-3">
      {games.map((g) => (
        <GameCard
          key={g.id}
          game={g}
          selected={g.id === selectedId}
          onSelect={() => onSelect(g.id === selectedId ? null : g.id)}
          onConfigure={() => onConfigure(g.id)}
          onLaunch={() => onLaunch(g.id)}
        />
      ))}
    </div>
  );
}
