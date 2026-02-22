import { create } from "zustand";
import { load } from "@tauri-apps/plugin-store";
import type { Settings } from "../lib/types";
import { commands } from "../lib/commands";

interface SettingsState {
  settings: Settings;
  loaded: boolean;
  showModal: boolean;
  setShowModal: (show: boolean) => void;
  loadSettings: () => Promise<void>;
  updateSettings: (partial: Partial<Settings>) => void;
  saveSettings: () => Promise<void>;
  resetWizard: () => void;
}

const defaultSettings: Settings = {
  anthropic_api_key: "",
  openai_api_key: "",
  openrouter_api_key: "",
  ollama_api_key: "",
  ollama_base_url: "http://127.0.0.1:11434",
  provider: "anthropic",
  model: "claude-sonnet-4-6",
  display_width: 1280,
  display_height: 800,
  shell_timeout_secs: 30,
  setup_complete: false,
};

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: defaultSettings,
  loaded: false,
  showModal: false,
  setShowModal: (show) => set({ showModal: show }),
  loadSettings: async () => {
    try {
      // Load from persistent store
      const store = await load("settings.json", { autoSave: true, defaults: {} });
      const saved = await store.get<Settings>("settings");
      if (saved) {
        const merged = { ...defaultSettings, ...saved };
        // Migration: auto-mark setup_complete if any API key is already set
        if (
          !merged.setup_complete &&
          (merged.anthropic_api_key ||
            merged.openai_api_key ||
            merged.openrouter_api_key ||
            merged.ollama_api_key)
        ) {
          merged.setup_complete = true;
        }
        set({ settings: merged, loaded: true });
        // Sync to Rust state
        await commands.saveSettings(merged);
        return;
      }
    } catch {
      // Ignore store errors
    }
    try {
      const settings = await commands.getSettings();
      set({ settings, loaded: true });
    } catch {
      set({ loaded: true });
    }
  },
  updateSettings: (partial) =>
    set((s) => ({ settings: { ...s.settings, ...partial } })),
  saveSettings: async () => {
    const settings = get().settings;
    try {
      // Save to Rust state
      await commands.saveSettings(settings);
      // Persist to store
      const store = await load("settings.json", { autoSave: true, defaults: {} });
      await store.set("settings", settings);
      await store.save();
    } catch (e) {
      console.error("Failed to save settings:", e);
    }
  },
  resetWizard: () => {
    set((s) => ({
      settings: { ...s.settings, setup_complete: false },
      showModal: false,
    }));
    // Persist the reset
    get().saveSettings();
  },
}));
