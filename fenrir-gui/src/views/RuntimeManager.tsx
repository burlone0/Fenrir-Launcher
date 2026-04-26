import { useState } from "react";
import { useRuntimesStore } from "../stores/runtimes";
import { useUIStore } from "../stores/ui";
import ProgressBar from "../components/ProgressBar";
import ConfirmDialog from "../components/ConfirmDialog";
import type { Runtime } from "../lib/types";

function RuntimeRow({
  runtime,
  onSetDefault,
}: {
  runtime: Runtime;
  onSetDefault: (id: string) => void;
}) {
  return (
    <tr className="border-t border-zinc-800">
      <td className="py-2.5 pr-4 font-mono text-xs text-zinc-300 align-middle">
        {runtime.version}
      </td>
      <td className="py-2.5 pr-4 text-xs text-zinc-400 align-middle">{runtime.runtime_type}</td>
      <td className="py-2.5 pr-4 text-xs text-zinc-400 align-middle">{runtime.source}</td>
      <td className="py-2.5 align-middle">
        {runtime.is_default ? (
          <span className="text-xs bg-green-900 text-green-200 px-1.5 py-0.5 rounded border border-green-800">
            Default
          </span>
        ) : (
          <button
            onClick={() => onSetDefault(runtime.id)}
            className="text-xs text-zinc-500 hover:text-white underline underline-offset-2"
          >
            Set default
          </button>
        )}
      </td>
    </tr>
  );
}

export default function RuntimeManager() {
  const { installed, available, isInstalling, downloadProgress, fetchAvailable, installRuntime, setDefault } =
    useRuntimesStore();
  const { notify } = useUIStore();

  const [kind, setKind] = useState<"proton-ge" | "wine-ge">("proton-ge");
  const [confirmInstall, setConfirmInstall] = useState<string | null>(null);

  const handleFetch = async () => {
    await fetchAvailable(kind);
    notify("Fetched available runtimes", "info");
  };

  const handleInstall = async (version: string) => {
    setConfirmInstall(null);
    await installRuntime(version);
    notify(`Installing ${version}…`, "info");
  };

  const handleSetDefault = async (id: string) => {
    await setDefault(id);
    const r = installed.find((x) => x.id === id);
    notify(`Default set to ${r?.version ?? id}`, "success");
  };

  return (
    <div className="h-full overflow-y-auto p-6 flex flex-col gap-8 max-w-3xl">

      {/* Installed */}
      <section>
        <h2 className="font-semibold mb-4">Installed runtimes</h2>
        {installed.length === 0 ? (
          <div className="flex flex-col gap-2 py-2">
            <p className="text-sm text-zinc-500">No runtimes installed.</p>
            <p className="text-xs text-zinc-600">
              Use <strong className="text-zinc-400">Fetch releases</strong> below to download GE-Proton or Wine-GE.
              At least one runtime is required to configure and launch games.
            </p>
          </div>
        ) : (
          <table className="w-full text-left">
            <thead>
              <tr className="text-[10px] text-zinc-500 uppercase tracking-wider">
                <th className="pb-2 pr-4">Version</th>
                <th className="pb-2 pr-4">Type</th>
                <th className="pb-2 pr-4">Source</th>
                <th className="pb-2">Default</th>
              </tr>
            </thead>
            <tbody>
              {installed.map((r) => (
                <RuntimeRow key={r.id} runtime={r} onSetDefault={handleSetDefault} />
              ))}
            </tbody>
          </table>
        )}
      </section>

      {/* Download progress */}
      {isInstalling && downloadProgress && (
        <ProgressBar
          value={downloadProgress.received}
          max={downloadProgress.total}
          label="Downloading runtime…"
        />
      )}

      {/* Available */}
      <section>
        <h2 className="font-semibold mb-4">Available runtimes</h2>
        <div className="flex gap-2 items-center mb-4">
          <select
            value={kind}
            onChange={(e) => setKind(e.target.value as typeof kind)}
            className="bg-zinc-800 border border-zinc-700 text-sm rounded px-3 py-1.5 text-zinc-200 focus:outline-none focus:border-sky-500"
          >
            <option value="proton-ge">GE-Proton</option>
            <option value="wine-ge">Wine-GE</option>
          </select>
          <button
            onClick={handleFetch}
            className="text-xs px-3 py-1.5 rounded bg-sky-700 hover:bg-sky-600 text-white"
          >
            Fetch releases
          </button>
        </div>

        {available.length === 0 ? (
          <p className="text-xs text-zinc-600">
            Click "Fetch releases" to load available versions from GitHub.
          </p>
        ) : (
          <div className="flex flex-col divide-y divide-zinc-800">
            {available.slice(0, 10).map((r) => (
              <div key={r.tag_name} className="flex items-center justify-between py-2">
                <span className="font-mono text-xs text-zinc-300">{r.tag_name}</span>
                <button
                  onClick={() => setConfirmInstall(r.tag_name)}
                  disabled={isInstalling}
                  className="text-xs px-3 py-1 rounded bg-zinc-700 hover:bg-zinc-600 text-white disabled:opacity-40"
                >
                  Install
                </button>
              </div>
            ))}
          </div>
        )}
      </section>

      {confirmInstall && (
        <ConfirmDialog
          title="Install runtime"
          message={`Download and install ${confirmInstall}? This may take a few minutes.`}
          confirmLabel="Install"
          onConfirm={() => handleInstall(confirmInstall)}
          onCancel={() => setConfirmInstall(null)}
        />
      )}
    </div>
  );
}
