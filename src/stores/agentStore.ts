import { create } from "zustand";
import type { ActionEntry, AgentStatus } from "../lib/types";

interface AgentState {
  status: AgentStatus;
  task: string;
  actions: ActionEntry[];
  thinking: string;
  message: string;
  estimatedSeconds: number | null;
  screenshot: string | null;
  cursorX: number;
  cursorY: number;

  setStatus: (status: AgentStatus) => void;
  setTask: (task: string) => void;
  addAction: (action: ActionEntry) => void;
  setThinking: (text: string) => void;
  setMessage: (text: string) => void;
  setEstimatedSeconds: (seconds: number | null) => void;
  setScreenshot: (base64: string | null) => void;
  setCursor: (x: number, y: number) => void;
  reset: () => void;
}

export const useAgentStore = create<AgentState>((set) => ({
  status: "idle",
  task: "",
  actions: [],
  thinking: "",
  message: "",
  estimatedSeconds: null,
  screenshot: null,
  cursorX: 0,
  cursorY: 0,

  setStatus: (status) => set({ status }),
  setTask: (task) => set({ task }),
  addAction: (action) =>
    set((s) => ({ actions: [...s.actions, action] })),
  setThinking: (thinking) => set({ thinking }),
  setMessage: (message) => set({ message }),
  setEstimatedSeconds: (estimatedSeconds) => set({ estimatedSeconds }),
  setScreenshot: (screenshot) => set({ screenshot }),
  setCursor: (cursorX, cursorY) => set({ cursorX, cursorY }),
  reset: () =>
    set({
      status: "idle",
      task: "",
      actions: [],
      thinking: "",
      message: "",
      estimatedSeconds: null,
      screenshot: null,
      cursorX: 0,
      cursorY: 0,
    }),
}));
