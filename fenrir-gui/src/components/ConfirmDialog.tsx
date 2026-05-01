interface Props {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  destructive?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export default function ConfirmDialog({
  title,
  message,
  confirmLabel = "Confirm",
  cancelLabel = "Cancel",
  destructive = false,
  onConfirm,
  onCancel,
}: Props) {
  return (
    <div className="fixed inset-0 bg-black/60 z-50 flex items-center justify-center">
      <div className="bg-zinc-900 border border-zinc-700 rounded-lg p-6 w-[400px] max-w-[90vw] flex flex-col gap-4">
        <h3 className="font-semibold text-base">{title}</h3>
        <p className="text-sm text-zinc-400">{message}</p>
        <div className="flex gap-2 justify-end">
          <button
            onClick={onCancel}
            className="text-xs px-4 py-2 rounded border border-zinc-700 text-zinc-300 hover:bg-zinc-800"
          >
            {cancelLabel}
          </button>
          <button
            onClick={onConfirm}
            className={`text-xs px-4 py-2 rounded text-white ${
              destructive
                ? "bg-red-700 hover:bg-red-600"
                : "bg-sky-700 hover:bg-sky-600"
            }`}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
