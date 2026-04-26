import { create } from "zustand";
import type { Game, ClassifiedGame } from "../lib/types";
import { listGames, confirmGame as confirmGameCmd, deleteGame as deleteGameCmd } from "../lib/commands";

interface GamesStore {
  games: Game[];
  selectedId: string | null;
  isLoading: boolean;
  error: string | null;

  loadGames: () => Promise<void>;
  selectGame: (id: string | null) => void;
  configureGame: (id: string, clean: boolean) => Promise<void>;
  launchGame: (id: string) => Promise<void>;
  deleteGame: (id: string) => Promise<void>;
  addDetectedGames: (games: ClassifiedGame[]) => void;
  updateGame: (game: Game) => void;
}

export const useGamesStore = create<GamesStore>((set, get) => ({
  games: [],
  selectedId: null,
  isLoading: false,
  error: null,

  loadGames: async () => {
    set({ isLoading: true, error: null });
    try {
      const games = await listGames();
      set({ games, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  selectGame: (id) => set({ selectedId: id }),

  configureGame: async (_id, _clean) => {
    // TODO Sprint 5: invoke configure_game + listen configure:done
  },

  launchGame: async (_id) => {
    // TODO Sprint 5: invoke launch_game + listen launch:ended
  },

  deleteGame: async (id) => {
    await deleteGameCmd(id);
    set((s) => ({
      games: s.games.filter((g) => g.id !== id),
      selectedId: s.selectedId === id ? null : s.selectedId,
    }));
  },

  addDetectedGames: (_classified) => {
    get().loadGames();
  },

  updateGame: (game) =>
    set((s) => ({ games: s.games.map((g) => (g.id === game.id ? game : g)) })),
}));

export { confirmGameCmd };
