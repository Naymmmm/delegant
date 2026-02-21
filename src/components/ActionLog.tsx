import { useEffect, useRef } from "react";
import { useAgentStore } from "../stores/agentStore";
import { ActionEntryRow } from "./ActionEntry";

export function ActionLog() {
  const actions = useAgentStore((s) => s.actions);
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [actions.length]);

  if (actions.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-zinc-600 text-xs">
        No actions yet...
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-0.5 overflow-y-auto h-full px-1 py-1">
      {actions.map((action) => (
        <ActionEntryRow key={action.id} entry={action} />
      ))}
      <div ref={bottomRef} />
    </div>
  );
}
