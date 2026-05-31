import { useState, useEffect } from "react";
import { getConfig, setConfig, listRuntimes } from "../lib/commands";
import { useUIStore } from "../stores/ui";
import ErrorBanner from "../components/ErrorBanner";
import type { FenrirConfig, Runtime } from "../lib/types";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function Field({
  label,
  value,
  mono,
}: {
  label: string;
  value: string;
  mono?: boolean;
}) {
  return (
    <div className="flex flex-col gap-0.5">
      <span className="text-zinc-500 uppercase text-[10px] tracking-wider">{label}</span>
      <span
        className={`text-zinc-200 break-all leading-snug ${
          mono ? "font-mono text-[10px]" : "text-xs"
        }`}
      >
        {value || "—"}
      </span>
    </div>
  );
}

function SectionHeader({ children }: { children: React.ReactNode }) {
  return (
    <h2 className="text-xs font-semibold uppercase tracking-wider text-zinc-500 mb-3">
      {children}
    </h2>
  );
}

function Toggle({
  checked,
  onChange,
  id,
}: {
  checked: boolean;
  onChange: (next: boolean) => void;
  id: string;
}) {
  return (
    <button
      id={id}
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      className={`relative inline-flex h-5 w-9 shrink-0 items-center rounded-full transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-sky-500 ${
        checked ? "bg-sky-600" : "bg-zinc-700"
      }`}
    >
      <span
        className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white shadow transition-transform ${
          checked ? "translate-x-[18px]" : "translate-x-1"
        }`}
      />
    </button>
  );
}

// ---------------------------------------------------------------------------
// Draft state — only the editable fields
// ---------------------------------------------------------------------------

interface Defaults {
  runtime: string;
  enable_dxvk: boolean;
  enable_vkd3d: boolean;
  esync: boolean;
  fsync: boolean;
}

function defaultsFromConfig(config: FenrirConfig): Defaults {
  return {
    runtime: config.defaults.runtime,
    enable_dxvk: config.defaults.enable_dxvk,
    enable_vkd3d: config.defaults.enable_vkd3d,
    esync: config.defaults.esync,
    fsync: config.defaults.fsync,
  };
}

function defaultsEqual(a: Defaults, b: Defaults): boolean {
  return (
    a.runtime === b.runtime &&
    a.enable_dxvk === b.enable_dxvk &&
    a.enable_vkd3d === b.enable_vkd3d &&
    a.esync === b.esync &&
    a.fsync === b.fsync
  );
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export default function Settings() {
  const { notify } = useUIStore();

  const [config, setConfigState] = useState<FenrirConfig | null>(null);
  const [runtimes, setRuntimes] = useState<Runtime[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);

  // Draft — tracks uncommitted edits to the Defaults section
  const [draft, setDraft] = useState<Defaults | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const [cfg, rtList] = await Promise.all([getConfig(), listRuntimes()]);
        if (!cancelled) {
          setConfigState(cfg);
          setRuntimes(rtList);
          setDraft(defaultsFromConfig(cfg));
        }
      } finally {
        if (!cancelled) setIsLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  if (isLoading || !config || !draft) {
    return (
      <div className="h-full flex items-center justify-center">
        <span className="text-zinc-500 text-sm">Loading…</span>
      </div>
    );
  }

  const saved = defaultsFromConfig(config);
  const isDirty = !defaultsEqual(draft, saved);

  const handleDiscard = () => setDraft({ ...saved });

  const handleSave = async () => {
    setSaveError(null);
    setIsSaving(true);
    try {
      const pairs: Array<[string, string]> = [];

      if (draft.runtime !== saved.runtime)
        pairs.push(["defaults.runtime", draft.runtime]);
      if (draft.enable_dxvk !== saved.enable_dxvk)
        pairs.push(["defaults.enable_dxvk", String(draft.enable_dxvk)]);
      if (draft.enable_vkd3d !== saved.enable_vkd3d)
        pairs.push(["defaults.enable_vkd3d", String(draft.enable_vkd3d)]);
      if (draft.esync !== saved.esync)
        pairs.push(["defaults.esync", String(draft.esync)]);
      if (draft.fsync !== saved.fsync)
        pairs.push(["defaults.fsync", String(draft.fsync)]);

      for (const [key, value] of pairs) {
        await setConfig(key, value);
      }

      // Refresh config from backend so saved state is authoritative
      const updated = await getConfig();
      setConfigState(updated);
      setDraft(defaultsFromConfig(updated));

      notify("Settings saved", "success");
    } catch (e) {
      setSaveError(String(e));
    } finally {
      setIsSaving(false);
    }
  };

  const setDraftField = <K extends keyof Defaults>(key: K, value: Defaults[K]) =>
    setDraft((prev) => (prev ? { ...prev, [key]: value } : prev));

  return (
    <div className="h-full overflow-y-auto p-6 flex flex-col gap-6 max-w-2xl">

      {/* ------------------------------------------------------------------ */}
      {/* Defaults (editable)                                                  */}
      {/* ------------------------------------------------------------------ */}
      <section className="bg-zinc-900 border border-zinc-800 rounded-lg p-5 flex flex-col gap-4">
        <SectionHeader>Defaults</SectionHeader>

        {/* Runtime dropdown */}
        <div className="flex flex-col gap-1.5">
          <label
            htmlFor="defaults-runtime"
            className="text-zinc-500 uppercase text-[10px] tracking-wider"
          >
            Runtime
          </label>
          <select
            id="defaults-runtime"
            value={draft.runtime}
            onChange={(e) => setDraftField("runtime", e.target.value)}
            className="bg-zinc-800 border border-zinc-700 text-xs rounded px-3 py-1.5 text-zinc-200 focus:outline-none focus:border-sky-500 w-full max-w-sm"
          >
            <option value="auto">Auto</option>
            {runtimes.map((rt) => (
              <option key={rt.id} value={rt.id}>
                {rt.version} ({rt.runtime_type})
              </option>
            ))}
          </select>
        </div>

        {/* Toggle rows */}
        {(
          [
            ["defaults-dxvk", "Enable DXVK", "enable_dxvk"],
            ["defaults-vkd3d", "Enable VKD3D", "enable_vkd3d"],
            ["defaults-esync", "ESync", "esync"],
            ["defaults-fsync", "FSync", "fsync"],
          ] as Array<[string, string, keyof Defaults]>
        ).map(([id, label, field]) => (
          <div key={id} className="flex items-center justify-between">
            <label htmlFor={id} className="text-zinc-400 text-xs cursor-pointer select-none">
              {label}
            </label>
            <Toggle
              id={id}
              checked={draft[field] as boolean}
              onChange={(next) => setDraftField(field, next)}
            />
          </div>
        ))}

        {/* Error */}
        {saveError && (
          <ErrorBanner message={saveError} onDismiss={() => setSaveError(null)} />
        )}

        {/* Action row */}
        <div className="flex gap-2 pt-1">
          <button
            onClick={handleSave}
            disabled={!isDirty || isSaving}
            className={`text-xs px-4 py-1.5 rounded text-white transition-colors ${
              !isDirty || isSaving
                ? "bg-sky-700 opacity-50 cursor-not-allowed"
                : "bg-sky-700 hover:bg-sky-600"
            }`}
          >
            {isSaving ? "Saving…" : "Save"}
          </button>
          {isDirty && !isSaving && (
            <button
              onClick={handleDiscard}
              className="text-xs px-4 py-1.5 rounded border border-zinc-700 text-zinc-400 hover:bg-zinc-800 transition-colors"
            >
              Discard changes
            </button>
          )}
        </div>
      </section>

      {/* ------------------------------------------------------------------ */}
      {/* Scan (read-only)                                                     */}
      {/* ------------------------------------------------------------------ */}
      <section className="bg-zinc-900 border border-zinc-800 rounded-lg p-5 flex flex-col gap-4">
        <SectionHeader>Scan</SectionHeader>

        <div className="flex flex-col gap-1.5">
          <span className="text-zinc-500 uppercase text-[10px] tracking-wider">Game directories</span>
          {config.scan.game_dirs.length === 0 ? (
            <span className="text-zinc-500 text-xs italic">No directories configured.</span>
          ) : (
            <ul className="flex flex-col gap-1">
              {config.scan.game_dirs.map((dir) => (
                <li key={dir} className="font-mono text-[10px] text-zinc-200 break-all">
                  {dir}
                </li>
              ))}
            </ul>
          )}
        </div>

        <Field
          label="Auto scan"
          value={config.scan.auto_scan ? "Enabled" : "Disabled"}
        />
      </section>

      {/* ------------------------------------------------------------------ */}
      {/* Privacy (read-only)                                                  */}
      {/* ------------------------------------------------------------------ */}
      <section className="bg-zinc-900 border border-zinc-800 rounded-lg p-5 flex flex-col gap-4">
        <SectionHeader>Privacy</SectionHeader>

        <Field
          label="Fetch metadata"
          value={config.privacy.fetch_metadata ? "Enabled" : "Disabled"}
        />
        <Field
          label="Fetch covers"
          value={config.privacy.fetch_covers ? "Enabled" : "Disabled"}
        />
        <Field label="Metadata source" value={config.privacy.metadata_source} />
      </section>

      {/* ------------------------------------------------------------------ */}
      {/* Paths (read-only)                                                    */}
      {/* ------------------------------------------------------------------ */}
      <section className="bg-zinc-900 border border-zinc-800 rounded-lg p-5 flex flex-col gap-4">
        <SectionHeader>Paths</SectionHeader>

        <Field label="Library database" value={config.general.library_db} mono />
        <Field label="Prefix directory" value={config.general.prefix_dir} mono />
        <Field label="Runtime directory" value={config.general.runtime_dir} mono />
      </section>

    </div>
  );
}
