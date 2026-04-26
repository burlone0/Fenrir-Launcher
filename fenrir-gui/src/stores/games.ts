import { create } from "zustand";
import type { Game, ClassifiedGame } from "../lib/types";
import {
  listGames,
  configureGame as configureGameCmd,
  launchGame as launchGameCmd,
  deleteGame as deleteGameCmd,
} from "../lib/commands";
import { onConfigureDone, onLaunchEnded } from "../lib/events";

interface GamesStore {
  games: Game[];
  selectedId: string | null;
  isLoading: boolean;
  error: string | null;
  configuringId: string | null;
  launchingId: string | null;

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
  configuringId: null,
  launchingId: null,

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

  configureGame: async (id, clean) => {
    set({ configuringId: id });
    let unlistenDone: (() => void) | null = null;
    try {
      const unlisten = await onConfigureDone((game) => {
        get().updateGame(game);
        unlisten();
        set({ configuringId: null });
      });
      unlistenDone = unlisten;
      await configureGameCmd(id, clean);
    } catch (e) {
      unlistenDone?.();
      set({ configuringId: null });
      throw e;
    }
  },

  launchGame: async (id) => {
    set({ launchingId: id });
    let unlistenEnded: (() => void) | null = null;
    try {
      const unlisten = await onLaunchEnded(({ game_id, play_time_secs }) => {
        set((s) => ({
          games: s.games.map((g) =>
            g.id === game_id
              ? {
                  ...g,
                  play_time: g.play_time + play_time_secs,
                  last_played: new Date().toISOString(),
                }
              : g
          ),
          launchingId: null,
        }));
        unlisten();
      });
      unlistenEnded = unlisten;
      await launchGameCmd(id);
    } catch (e) {
      unlistenEnded?.();
      set({ launchingId: null });
      throw e;
    }
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
