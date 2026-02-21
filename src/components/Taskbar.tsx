import { useAgentStore } from "../stores/agentStore";
import { useUIStore } from "../stores/uiStore";
import { ChevronUp } from "lucide-react";
import { StopButton } from "./StopButton";
import { Spinner } from "./Spinner";

export function Taskbar() {
  const task = useAgentStore((s) => s.task);
  const thinking = useAgentStore((s) => s.thinking);
  const { toggleExpanded } = useUIStore();

  const truncatedTask = task.length > 36 ? task.slice(0, 36) + "…" : task;

  return (
    <div
      className="flex items-center gap-3 h-full px-4 bg-zinc-900 border-t border-white/10 hover:bg-zinc-800 cursor-pointer transition-colors"
      onClick={toggleExpanded}
    >
      {/* Pulsing activity indicator */}
      <div className="relative shrink-0">
        <div className="w-2 h-2 rounded-full bg-blue-400" />
        <div className="absolute inset-0 w-2 h-2 rounded-full bg-blue-400 animate-ping opacity-60" />
      </div>

      {/* Task label */}
      <div className="flex-1 min-w-0">
        <p className="text-[11px] text-zinc-100 font-medium truncate leading-tight">
          {truncatedTask}
        </p>
        {thinking && (
          <p className="text-[10px] text-zinc-400 truncate leading-tight mt-0.5">
            {thinking}
          </p>
        )}
      </div>

      {/* Status spinner */}
      <Spinner size={14} className="text-blue-400 shrink-0" />

      <ChevronUp className="w-4 h-4 text-zinc-400 shrink-0" />

      {/* Stop button */}
      <div onClick={(e) => e.stopPropagation()}>
        <StopButton />
      </div>
    </div>
  );
}
