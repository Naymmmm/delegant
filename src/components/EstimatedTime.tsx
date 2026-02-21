import { useAgentStore } from "../stores/agentStore";

export function EstimatedTime() {
  const seconds = useAgentStore((s) => s.estimatedSeconds);
  const thinking = useAgentStore((s) => s.thinking);

  if (thinking) {
    return <span className="text-xs text-zinc-500">Thinking...</span>;
  }

  if (seconds === null) {
    return <span className="text-xs text-zinc-500">Working...</span>;
  }

  if (seconds < 60) {
    return <span className="text-xs text-zinc-500">~{seconds}s remaining</span>;
  }

  const minutes = Math.ceil(seconds / 60);
  return (
    <span className="text-xs text-zinc-500">~{minutes}m remaining</span>
  );
}
