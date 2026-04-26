import { useEffect } from "react";
import { useUIStore } from "./stores/ui";
import { useGamesStore } from "./stores/games";
import { useRuntimesStore } from "./stores/runtimes";
import Library from "./views/Library";
import RuntimeManager from "./views/RuntimeManager";

function Sidebar() {
  const { currentView, navigate } = useUIStore();

  const navItem = (view: Parameters<typeof navigate>[0], label: string) => (
    <button
      onClick={() => navigate(view)}
      className={`w-full text-left px-4 py-2 rounded text-sm font-medium transition-colors ${
        currentView === view
          ? "bg-zinc-700 text-white"
          : "text-zinc-400 hover:text-white hover:bg-zinc-800"
      }`}
    >
      {label}
    </button>
  );

  return (
    <aside className="w-48 shrink-0 bg-zinc-900 border-r border-zinc-800 flex flex-col py-4 px-2 gap-1">
      <div className="px-4 pb-4 mb-2 border-b border-zinc-800">
        <span className="text-white font-bold text-lg tracking-tight">Fenrir</span>
      </div>
      {navItem("library", "Library")}
      {navItem("runtimes", "Runtimes")}
      {navItem("settings", "Settings")}
    </aside>
  );
}

function Notification() {
  const { notification, clearNotification } = useUIStore();
  if (!notification) return null;

  const color =
    notification.type === "error"
      ? "bg-red-900 border-red-700 text-red-200"
      : notification.type === "success"
      ? "bg-green-900 border-green-700 text-green-200"
      : "bg-zinc-800 border-zinc-600 text-zinc-200";

  return (
    <div
      className={`fixed bottom-4 right-4 z-50 px-4 py-3 rounded border text-sm flex items-center gap-3 ${color}`}
    >
      <span>{notification.message}</span>
      <button onClick={clearNotification} className="opacity-60 hover:opacity-100">
        ✕
      </button>
    </div>
  );
}

export default function App() {
  const { currentView } = useUIStore();
  const loadGames = useGamesStore((s) => s.loadGames);
  const loadInstalled = useRuntimesStore((s) => s.loadInstalled);

  useEffect(() => {
    loadGames();
    loadInstalled();
  }, [loadGames, loadInstalled]);

  return (
    <div className="flex h-screen bg-zinc-950 text-white overflow-hidden">
      <Sidebar />
      <main className="flex-1 overflow-hidden">
        {currentView === "library" && <Library />}
        {currentView === "runtimes" && <RuntimeManager />}
        {currentView === "settings" && (
          <div className="p-8 text-zinc-400">Settings — post-MVP</div>
        )}
      </main>
      <Notification />
    </div>
  );
}
