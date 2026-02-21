import { create } from "zustand";
import type { UIMode } from "../lib/types";

export interface Toast {
  id: string;
  message: string;
  type: "error" | "info" | "success";
}

interface UIState {
  mode: UIMode;
  expanded: boolean;
  toasts: Toast[];
  setMode: (mode: UIMode) => void;
  setExpanded: (expanded: boolean) => void;
  toggleExpanded: () => void;
  addToast: (message: string, type: Toast["type"]) => void;
  removeToast: (id: string) => void;
}

export const useUIStore = create<UIState>((set) => ({
  mode: "start",
  expanded: false,
  toasts: [],
  setMode: (mode) => set({ mode, expanded: false }),
  setExpanded: (expanded) => set({ expanded }),
  toggleExpanded: () => set((s) => ({ expanded: !s.expanded })),
  addToast: (message, type) => {
    const id = crypto.randomUUID();
    set((s) => ({ toasts: [...s.toasts, { id, message, type }] }));
    setTimeout(() => {
      set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) }));
    }, 4000);
  },
  removeToast: (id) =>
    set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })),
}));
