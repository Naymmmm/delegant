import { useEffect, useState } from "react";
import { X, AlertCircle, Info, CheckCircle } from "lucide-react";
import { clsx } from "clsx";
import { useUIStore } from "../stores/uiStore";
import type { Toast } from "../stores/uiStore";

const icons: Record<Toast["type"], React.ReactNode> = {
  error: <AlertCircle className="w-4 h-4 text-red-400 shrink-0" />,
  info: <Info className="w-4 h-4 text-blue-400 shrink-0" />,
  success: <CheckCircle className="w-4 h-4 text-green-400 shrink-0" />,
};

const borders: Record<Toast["type"], string> = {
  error: "border-red-500/30",
  info: "border-blue-500/30",
  success: "border-green-500/30",
};

function ToastItem({ toast }: { toast: Toast }) {
  const removeToast = useUIStore((s) => s.removeToast);
  const [exiting, setExiting] = useState(false);

  useEffect(() => {
    const timer = setTimeout(() => setExiting(true), 3600);
    return () => clearTimeout(timer);
  }, []);

  return (
    <div
      className={clsx(
        "flex items-center gap-2.5 px-4 py-2.5 rounded-xl bg-zinc-900 border shadow-lg shadow-black/30 max-w-sm",
        borders[toast.type],
        exiting ? "animate-toast-out" : "animate-toast-in"
      )}
      onAnimationEnd={() => {
        if (exiting) removeToast(toast.id);
      }}
    >
      {icons[toast.type]}
      <p className="text-sm text-zinc-200 flex-1">{toast.message}</p>
      <button
        onClick={() => removeToast(toast.id)}
        className="p-0.5 text-zinc-500 hover:text-zinc-300 transition-colors"
      >
        <X className="w-3.5 h-3.5" />
      </button>
    </div>
  );
}

export function ToastContainer() {
  const toasts = useUIStore((s) => s.toasts);

  if (toasts.length === 0) return null;

  return (
    <div className="fixed top-4 right-4 z-[100] flex flex-col gap-2">
      {toasts.map((t) => (
        <ToastItem key={t.id} toast={t} />
      ))}
    </div>
  );
}
