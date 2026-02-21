import { Square } from "lucide-react";
import { commands } from "../lib/commands";
import { restoreDefaultWindow } from "../lib/windowManager";
import { useAgentStore } from "../stores/agentStore";
import { useUIStore } from "../stores/uiStore";

export function StopButton() {
  const reset = useAgentStore((s) => s.reset);
  const setMode = useUIStore((s) => s.setMode);

  const handleStop = async () => {
    try {
      await commands.stopAgent();
    } catch (e) {
      console.error("Failed to stop agent:", e);
    }
    reset();
    await restoreDefaultWindow();
    setMode("start");
  };

  return (
    <button
      onClick={handleStop}
      className="flex items-center justify-center w-8 h-8 rounded-full bg-red-600 hover:bg-red-500 transition-colors shadow-lg shadow-red-600/20 btn-press"
      title="Stop agent"
    >
      <Square className="w-3.5 h-3.5 text-white fill-white" />
    </button>
  );
}
