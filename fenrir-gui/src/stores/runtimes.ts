import { create } from "zustand";
import type { Runtime, GitHubRelease } from "../lib/types";
import { MOCK_RUNTIMES } from "../lib/mock";

interface RuntimesStore {
  installed: Runtime[];
  available: GitHubRelease[];
  isInstalling: boolean;
  downloadProgress: { received: number; total: number } | null;

  loadInstalled: () => Promise<void>;
  fetchAvailable: (kind: "proton-ge" | "wine-ge") => Promise<void>;
  installRuntime: (version: string) => Promise<void>;
  setDefault: (id: string) => Promise<void>;
}

export const useRuntimesStore = create<RuntimesStore>((set) => ({
  installed: [],
  available: [],
  isInstalling: false,
  downloadProgress: null,

  loadInstalled: async () => {
    // TODO Sprint 4: replace with listRuntimes()
    set({ installed: MOCK_RUNTIMES });
  },

  fetchAvailable: async (_kind) => {
    // TODO Sprint 4: replace with availableRuntimes(kind)
    set({ available: [] });
  },

  installRuntime: async (_version) => {
    // TODO Sprint 5: invoke install_runtime + listen download:progress/done
    set({ isInstalling: false, downloadProgress: null });
  },

  setDefault: async (id) => {
    // TODO Sprint 4: invoke setDefaultRuntime(id)
    set((s) => ({
      installed: s.installed.map((r) => ({
        ...r,
        is_default: r.id === id,
      })),
    }));
  },
}));
