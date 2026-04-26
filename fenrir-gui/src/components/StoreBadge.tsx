import type { StoreOrigin } from "../lib/types";

const CONFIG: Record<StoreOrigin, { classes: string }> = {
  Steam:   { classes: "bg-sky-800 text-sky-100" },
  GOG:     { classes: "bg-purple-800 text-purple-100" },
  Epic:    { classes: "bg-orange-800 text-orange-100" },
  Unknown: { classes: "bg-zinc-700 text-zinc-300" },
};

export default function StoreBadge({ store }: { store: StoreOrigin }) {
  const { classes } = CONFIG[store];
  return (
    <span className={`inline-block text-xs px-1.5 py-0.5 rounded font-medium ${classes}`}>
      {store}
    </span>
  );
}
