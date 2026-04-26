import { create } from "zustand";
import type { Runtime, GitHubRelease } from "../lib/types";
import {
  listRuntimes,
  availableRuntimes,
  installRuntime as installRuntimeCmd,
  setDefaultRuntime,
} from "../lib/commands";
import { onDownloadProgress, onDownloadDone } from "../lib/events";

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
    try {
      const installed = await listRuntimes();
      set({ installed });
    } catch {
      set({ installed: [] });
    }
  },

  fetchAvailable: async (kind) => {
    try {
      const available = await availableRuntimes(kind);
      set({ available });
    } catch (e) {
      console.error("fetchAvailable failed:", e);
    }
  },

  installRuntime: async (version) => {
    set({ isInstalling: true, downloadProgress: null });

    const unlistenProgress = await onDownloadProgress(({ bytes_received, total_bytes }) => {
      set({ downloadProgress: { received: bytes_received, total: total_bytes } });
    });

    const unlistenDone = await onDownloadDone(() => {
      set({ isInstalling: false, downloadProgress: null });
      unlistenProgress();
      unlistenDone();
    });

    await installRuntimeCmd(version);
  },

  setDefault: async (id) => {
    await setDefaultRuntime(id);
    set((s) => ({
      installed: s.installed.map((r) => ({ ...r, is_default: r.id === id })),
    }));
  },
}));
