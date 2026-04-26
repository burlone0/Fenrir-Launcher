import { useRuntimesStore } from "../stores/runtimes";
import type { Runtime } from "../lib/types";

function RuntimeRow({ runtime }: { runtime: Runtime }) {
  const { setDefault } = useRuntimesStore();

  return (
    <tr className="border-t border-zinc-800 text-sm">
      <td className="py-2 pr-4 font-mono text-xs text-zinc-300">{runtime.version}</td>
      <td className="py-2 pr-4 text-zinc-400">{runtime.runtime_type}</td>
      <td className="py-2 pr-4 text-zinc-400">{runtime.source}</td>
      <td className="py-2 pr-4">
        {runtime.is_default ? (
          <span className="text-xs bg-green-800 text-green-200 px-1.5 py-0.5 rounded">
            Default
          </span>
        ) : (
          <button
            onClick={() => setDefault(runtime.id)}
            className="text-xs text-zinc-500 hover:text-white"
          >
            Set default
          </button>
        )}
      </td>
    </tr>
  );
}

export default function RuntimeManager() {
  const { installed } = useRuntimesStore();

  return (
    <div className="h-full overflow-y-auto p-6 flex flex-col gap-8">
      <section>
        <h2 className="font-semibold mb-4">Installed runtimes</h2>
        {installed.length === 0 ? (
          <p className="text-zinc-500 text-sm">
            No runtimes installed. Install GE-Proton or Wine-GE below.
          </p>
        ) : (
          <table className="w-full text-left">
            <thead>
              <tr className="text-xs text-zinc-500 uppercase tracking-wider">
                <th className="pb-2 pr-4">Version</th>
                <th className="pb-2 pr-4">Type</th>
                <th className="pb-2 pr-4">Source</th>
                <th className="pb-2 pr-4">Default</th>
              </tr>
            </thead>
            <tbody>
              {installed.map((r) => (
                <RuntimeRow key={r.id} runtime={r} />
              ))}
            </tbody>
          </table>
        )}
      </section>

      <section>
        <h2 className="font-semibold mb-4">Available runtimes</h2>
        <div className="flex gap-3 items-center">
          <select className="bg-zinc-800 border border-zinc-700 text-sm rounded px-3 py-1.5 text-zinc-200">
            <option value="proton-ge">GE-Proton</option>
            <option value="wine-ge">Wine-GE</option>
          </select>
          <button className="text-xs px-3 py-1.5 rounded bg-sky-700 hover:bg-sky-600 text-white">
            Fetch
          </button>
        </div>
        <p className="text-zinc-500 text-xs mt-3">
          Sprint 4 — GitHub release fetch coming soon.
        </p>
      </section>
    </div>
  );
}
