import { create } from "zustand";
import type { ScanProgress, ScanDonePayload } from "../lib/types";

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

  scanDirectory: async (_path) => {
    // TODO Sprint 5: invoke scan_directory + listen scan:progress/done
    set({ isScanning: true, progress: null, lastResult: null });
  },

  confirmGame: async (_query) => {
    // TODO Sprint 4: invoke confirmGame(query)
  },

  clearResult: () => set({ lastResult: null, progress: null }),

  setProgress: (p) => set({ progress: p }),

  setResult: (r) => set({ lastResult: r, isScanning: false }),
}));
