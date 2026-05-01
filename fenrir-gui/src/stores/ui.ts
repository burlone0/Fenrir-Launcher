import { create } from "zustand";

type View = "library" | "scan" | "runtimes" | "settings";

interface UIStore {
  currentView: View;
  isScanOpen: boolean;
  notification: { message: string; type: "info" | "error" | "success" } | null;

  navigate: (view: View) => void;
  openScan: () => void;
  closeScan: () => void;
  notify: (message: string, type: "info" | "error" | "success") => void;
  clearNotification: () => void;
}

export const useUIStore = create<UIStore>((set) => ({
  currentView: "library",
  isScanOpen: false,
  notification: null,

  navigate: (view) => set({ currentView: view }),
  openScan: () => set({ isScanOpen: true }),
  closeScan: () => set({ isScanOpen: false }),
  notify: (message, type) => set({ notification: { message, type } }),
  clearNotification: () => set({ notification: null }),
}));
