import { invoke } from "@tauri-apps/api/core";
import type { Settings, CaptureResult, ShellResult, WindowInfo } from "./types";

export const commands = {
  takeScreenshot: () => invoke<CaptureResult>("take_screenshot"),

  mouseMove: (x: number, y: number) => invoke("mouse_move", { x, y }),
  mouseClick: (x: number, y: number, button: string) =>
    invoke("mouse_click", { x, y, button }),
  mouseDoubleClick: (x: number, y: number) =>
    invoke("mouse_double_click", { x, y }),
  mouseScroll: (x: number, y: number, clicks: number) =>
    invoke("mouse_scroll", { x, y, clicks }),
  mouseDrag: (startX: number, startY: number, endX: number, endY: number) =>
    invoke("mouse_drag", {
      startX,
      startY,
      endX,
      endY,
    }),

  keyPress: (combo: string) => invoke("key_press", { combo }),
  typeText: (text: string) => invoke("type_text", { text }),

  runShell: (command: string) => invoke<ShellResult>("run_shell", { command }),

  listWindows: () => invoke<WindowInfo[]>("list_windows"),
  focusWindow: (handle: number) => invoke("focus_window", { handle }),

  startAgent: (task: string) => invoke("start_agent", { task }),
  stopAgent: () => invoke("stop_agent"),

  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke("save_settings", { settings }),
};
