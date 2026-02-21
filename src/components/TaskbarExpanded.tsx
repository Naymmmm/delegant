import { ChevronDown } from "lucide-react";
import { useAgentStore } from "../stores/agentStore";
import { useUIStore } from "../stores/uiStore";
import { StopButton } from "./StopButton";
import { EstimatedTime } from "./EstimatedTime";
import { ActionLog } from "./ActionLog";
import { Spinner } from "./Spinner";
import { expandTaskbar, collapseTaskbar } from "../lib/windowManager";
import { useEffect } from "react";

export function TaskbarExpanded() {
  const task = useAgentStore((s) => s.task);
  const message = useAgentStore((s) => s.message);
  const thinking = useAgentStore((s) => s.thinking);
  const { setExpanded } = useUIStore();

  const truncatedTask = task.length > 45 ? task.slice(0, 45) + "…" : task;

  // Resize window when expanded
  useEffect(() => {
    expandTaskbar();
    return () => {
      collapseTaskbar();
    };
  }, []);

  return (
    <div className="flex flex-col h-full bg-zinc-900 border-t border-white/10 overflow-hidden">
      {/* Header */}
      <div className="flex items-center gap-2.5 px-4 py-3 border-b border-white/10 shrink-0 bg-zinc-900/50">
        {/* Activity indicator */}
        <div className="relative shrink-0">
          <div className="w-2 h-2 rounded-full bg-blue-400" />
          <div className="absolute inset-0 w-2 h-2 rounded-full bg-blue-400 animate-ping opacity-60" />
        </div>

        {/* Task info */}
        <div className="flex-1 min-w-0">
          <p className="text-[12px] text-zinc-100 font-medium truncate leading-tight">
            {truncatedTask}
          </p>
          <EstimatedTime />
        </div>

        {/* Collapse */}
        <button
          onClick={() => setExpanded(false)}
          className="p-1.5 rounded-md text-zinc-400 hover:text-zinc-200 hover:bg-white/[0.06] transition-colors"
        >
          <ChevronDown className="w-4 h-4" />
        </button>

        {/* Stop */}
        <StopButton />
      </div>

      {/* Thinking indicator */}
      {thinking && (
        <div className="flex items-center gap-2 px-4 py-2 border-b border-white/5 bg-zinc-800/20">
          <Spinner size={12} className="text-blue-400 shrink-0" />
          <p className="text-[11px] text-zinc-400 truncate">{thinking}</p>
        </div>
      )}

      {/* Action log */}
      <div className="flex-1 overflow-hidden">
        <ActionLog />
      </div>

      {/* Latest message */}
      {message && (
        <div className="px-4 py-3 border-t border-white/10 shrink-0 bg-zinc-900/50">
          <p className="text-[11px] text-zinc-400 leading-relaxed line-clamp-2">
            {message}
          </p>
        </div>
      )}
    </div>
  );
}
