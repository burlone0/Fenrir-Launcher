import { useEffect, useState } from "react";
import { Sun, Moon, Monitor } from "lucide-react";
import { useUIStore, type Theme } from "./stores/ui";
import { useGamesStore } from "./stores/games";
import { useRuntimesStore } from "./stores/runtimes";
import Library from "./views/Library";
import RuntimeManager from "./views/RuntimeManager";
import Settings from "./views/Settings";
import OnboardingWizard from "./components/OnboardingWizard";

const THEME_ICONS: Record<Theme, React.ReactNode> = {
  dark: <Moon size={14} />,
  light: <Sun size={14} />,
  system: <Monitor size={14} />,
};

const THEME_CYCLE: Theme[] = ["dark", "light", "system"];

function Sidebar() {
  const { currentView, navigate, theme, setTheme } = useUIStore();

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

  const cycleTheme = () => {
    const idx = THEME_CYCLE.indexOf(theme);
    setTheme(THEME_CYCLE[(idx + 1) % THEME_CYCLE.length]);
  };

  const themeLabel: Record<Theme, string> = {
    dark: "Dark",
    light: "Light",
    system: "System",
  };

  return (
    <aside className="w-48 shrink-0 bg-zinc-900 border-r border-zinc-800 flex flex-col py-4 px-2 gap-1">
      <div className="px-4 pb-4 mb-2 border-b border-zinc-800">
        <span className="text-white font-bold text-lg tracking-tight">Fenrir</span>
      </div>
      {navItem("library", "Library")}
      {navItem("runtimes", "Runtimes")}
      {navItem("settings", "Settings")}

      <div className="mt-auto pt-2 border-t border-zinc-800 px-2">
        <button
          onClick={cycleTheme}
          title={`Theme: ${themeLabel[theme]} — click to cycle`}
          className="w-full flex items-center gap-2 px-2 py-1.5 rounded text-xs text-zinc-400 hover:text-white hover:bg-zinc-800 transition-colors"
        >
          {THEME_ICONS[theme]}
          <span>{themeLabel[theme]}</span>
        </button>
      </div>
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
  const { currentView, theme } = useUIStore();
  const loadGames = useGamesStore((s) => s.loadGames);
  const loadInstalled = useRuntimesStore((s) => s.loadInstalled);
  const [showOnboarding, setShowOnboarding] = useState(false);

  // Apply dark/light class to <html> based on theme store value
  useEffect(() => {
    const apply = () => {
      const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      const isDark = theme === "dark" || (theme === "system" && prefersDark);
      document.documentElement.classList.toggle("dark", isDark);
    };

    apply();

    if (theme === "system") {
      const mql = window.matchMedia("(prefers-color-scheme: dark)");
      mql.addEventListener("change", apply);
      return () => mql.removeEventListener("change", apply);
    }
  }, [theme]);

  useEffect(() => {
    Promise.all([loadGames(), loadInstalled()]).then(() => {
      if (localStorage.getItem("fenrir.onboarding_done")) return;
      const games = useGamesStore.getState().games;
      const installed = useRuntimesStore.getState().installed;
      if (games.length === 0 && installed.length === 0) {
        setShowOnboarding(true);
      }
    });
  }, [loadGames, loadInstalled]);

  return (
    <div className="flex h-screen bg-zinc-950 text-white overflow-hidden">
      <Sidebar />
      <main className="flex-1 overflow-hidden">
        {currentView === "library" && <Library />}
        {currentView === "runtimes" && <RuntimeManager />}
        {currentView === "settings" && <Settings />}
      </main>
      <Notification />
      {showOnboarding && <OnboardingWizard onDone={() => setShowOnboarding(false)} />}
    </div>
  );
}
