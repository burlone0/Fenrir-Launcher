import { create } from "zustand";
import type { ScanProgress, ScanDonePayload } from "../lib/types";
import { scanDirectory as scanDirectoryCmd, confirmGame } from "../lib/commands";
import { useGamesStore } from "./games";

interface ScanStore {
  isScanning: boolean;
  progress: ScanProgress | null;
  lastResult: ScanDonePayload | null;

  scanDirectory: (path?: string) => Promise<void>;
  confirmGame: (query: string) => Promise<void>;
  clearResult: () => void;
  setProgress: (p: ScanProgress) => void;
  setResult: (r: ScanDonePayload) => void;
}

export const useScanStore = create<ScanStore>((set) => ({
  isScanning: false,
  progress: null,
  lastResult: null,

  scanDirectory: async (path) => {
    set({ isScanning: true, progress: null, lastResult: null });
    try {
      // Sprint 4: sync call returns ScanDonePayload directly
      // Sprint 5: will switch to events (scan:progress / scan:done)
      const result = await scanDirectoryCmd(path);
      set({
        isScanning: false,
        lastResult: result as unknown as ScanDonePayload,
      });
      // Refresh library after scan
      await useGamesStore.getState().loadGames();
    } catch (e) {
      set({ isScanning: false });
      throw e;
    }
  },

  confirmGame: async (query) => {
    await confirmGame(query);
    await useGamesStore.getState().loadGames();
  },

  clearResult: () => set({ lastResult: null, progress: null }),

  setProgress: (p) => set({ progress: p }),

  setResult: (r) => set({ lastResult: r, isScanning: false }),
}));
