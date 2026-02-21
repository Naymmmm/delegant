import {
  MousePointer2,
  Terminal,
  FileText,
  Clock,
} from "lucide-react";
import type { ActionEntry as ActionEntryType } from "../lib/types";

const iconMap: Record<string, React.ReactNode> = {
  computer: <MousePointer2 className="w-3.5 h-3.5 text-blue-400" />,
  bash: <Terminal className="w-3.5 h-3.5 text-green-400" />,
  text_editor: <FileText className="w-3.5 h-3.5 text-yellow-400" />,
};

function getRelativeTime(timestamp: string): string {
  const diff = Date.now() - new Date(timestamp).getTime();
  const seconds = Math.floor(diff / 1000);
  if (seconds < 5) return "just now";
  if (seconds < 60) return `${seconds}s ago`;
  const minutes = Math.floor(seconds / 60);
  return `${minutes}m ago`;
}

export function ActionEntryRow({ entry }: { entry: ActionEntryType }) {
  const icon = iconMap[entry.action_type] ?? (
    <Clock className="w-3.5 h-3.5 text-zinc-400" />
  );

  return (
    <div className="flex items-start gap-2.5 px-3 py-2 hover:bg-zinc-800/40 rounded-lg transition-colors animate-fade-in">
      <div className="mt-0.5 shrink-0">{icon}</div>
      <div className="flex-1 min-w-0">
        <p className="text-xs text-zinc-300 truncate">{entry.description}</p>
        {entry.output && (
          <p className="text-xs text-zinc-500 mt-0.5 truncate font-mono">
            {entry.output}
          </p>
        )}
      </div>
      <span className="text-xs text-zinc-600 shrink-0 mt-0.5">
        {getRelativeTime(entry.timestamp)}
      </span>
    </div>
  );
}
