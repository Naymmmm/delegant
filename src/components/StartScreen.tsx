import { useState, useEffect, useMemo, useRef } from "react";
import {
  Settings,
  Play,
  Sparkles,
  ChevronDown,
  Search,
  Loader2,
  FileCode2,
  Image as ImageIcon,
  Globe,
  FileText,
} from "lucide-react";
import { clsx } from "clsx";
import { commands } from "../lib/commands";
import { shrinkToTaskbar } from "../lib/windowManager";
import { PROVIDERS, MODELS, fetchModelsDetailed } from "../lib/types";
import { useSettingsStore } from "../stores/settingsStore";
import { useAgentStore } from "../stores/agentStore";
import { useUIStore } from "../stores/uiStore";
import { SettingsModal } from "./SettingsModal";
import { Spinner } from "./Spinner";

export function StartScreen() {
  const [task, setTask] = useState("");
  const [loading, setLoading] = useState(false);
  const [modelPickerOpen, setModelPickerOpen] = useState(false);
  const [modelSearch, setModelSearch] = useState("");
  const [loadingModels, setLoadingModels] = useState(false);
  const [modelsError, setModelsError] = useState(false);
  const pickerRef = useRef<HTMLDivElement>(null);
  const { settings, updateSettings, saveSettings, showModal, setShowModal } =
    useSettingsStore();
  const { setTask: setAgentTask } = useAgentStore();
  const { setMode, addToast } = useUIStore();

  const [dynamicModels, setDynamicModels] = useState(
    MODELS[settings.provider] ?? [],
  );

  useEffect(() => {
    const key =
      settings.provider === "anthropic"
        ? settings.anthropic_api_key
        : settings.provider === "openrouter"
          ? settings.openrouter_api_key
          : settings.provider === "ollama"
            ? settings.ollama_api_key
            : settings.openai_api_key;
    setDynamicModels([]);
    setModelsError(false);
    setLoadingModels(true);
    fetchModelsDetailed(settings.provider, key, settings.ollama_base_url)
      .then((result) => {
        setDynamicModels(result.models);
        setModelsError(result.hadError);
      })
      .finally(() => setLoadingModels(false));
  }, [
    settings.provider,
    settings.anthropic_api_key,
    settings.openai_api_key,
    settings.openrouter_api_key,
    settings.ollama_api_key,
    settings.ollama_base_url,
  ]);

  // Close picker on outside click
  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (pickerRef.current && !pickerRef.current.contains(e.target as Node)) {
        setModelPickerOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  const filteredModels = useMemo(() => {
    if (!modelSearch.trim()) return dynamicModels;
    const q = modelSearch.toLowerCase();
    return dynamicModels.filter(
      (m) =>
        m.id.toLowerCase().includes(q) || m.label.toLowerCase().includes(q),
    );
  }, [dynamicModels, modelSearch]);

  const currentModelLabel =
    dynamicModels.find((m) => m.id === settings.model)?.label ??
    settings.model ??
    "Select model";

  const handleSubmit = async () => {
    if (!task.trim()) return;

    const apiKey =
      settings.provider === "anthropic"
        ? settings.anthropic_api_key
        : settings.provider === "openrouter"
          ? settings.openrouter_api_key
          : settings.provider === "ollama"
            ? settings.ollama_api_key
            : settings.openai_api_key;

    if (!apiKey && settings.provider !== "ollama") {
      addToast("Please set your API key in settings first.", "error");
      setShowModal(true);
      return;
    }

    setLoading(true);

    try {
      await saveSettings();
      setAgentTask(task.trim());
      await commands.startAgent(task.trim());
      await shrinkToTaskbar();
      setMode("taskbar");
    } catch (e) {
      addToast(String(e), "error");
      setLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  };

  return (
    <div className="flex flex-col items-center justify-center h-full bg-zinc-950 px-8 select-none">
      {/* Header */}
      <div className="flex items-center gap-3 mb-8 animate-fade-in">
        <Sparkles className="w-8 h-8 text-blue-400" />
        <h1 className="text-2xl font-semibold text-white tracking-tight">
          Delegant
        </h1>
      </div>

      {/* Task Input */}
      <div className="w-full max-w-lg animate-slide-up">
        <textarea
          value={task}
          onChange={(e) => setTask(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="What should I do?"
          rows={3}
          className="w-full bg-zinc-900 border border-zinc-800 rounded-xl px-4 py-3 text-white text-base placeholder:text-zinc-500 focus:outline-none focus:ring-2 focus:ring-blue-500/40 focus:border-blue-500/40 resize-none input-glow"
          autoFocus
        />
      </div>

      {/* Provider & Model */}
      <div
        className="flex items-center gap-3 mt-4 animate-slide-up relative z-10"
        style={{ animationDelay: "0.05s" }}
      >
        <div className="flex bg-zinc-900 rounded-full p-0.5 border border-zinc-800">
          {PROVIDERS.map((p) => (
            <button
              key={p.id}
              onClick={() => {
                updateSettings({
                  provider: p.id,
                  model: (MODELS[p.id] ?? [])[0]?.id ?? "",
                });
                setModelPickerOpen(false);
                setModelSearch("");
              }}
              className={clsx(
                "px-4 py-1.5 rounded-full text-sm font-medium transition-all btn-press",
                settings.provider === p.id
                  ? "bg-zinc-800 text-white shadow-sm"
                  : "text-zinc-400 hover:text-zinc-200",
              )}
            >
              {p.label}
            </button>
          ))}
        </div>

        {/* Model picker button */}
        <div className="relative" ref={pickerRef}>
          <button
            onClick={() => {
              setModelPickerOpen(!modelPickerOpen);
              setModelSearch("");
            }}
            className="flex items-center gap-1.5 bg-zinc-900 border border-zinc-800 rounded-lg px-3 py-1.5 text-sm text-zinc-300 hover:text-white hover:border-zinc-700 transition-colors max-w-[200px]"
          >
            <span className="truncate">{currentModelLabel}</span>
            {loadingModels ? (
              <Loader2 className="w-3 h-3 shrink-0 animate-spinner text-zinc-500" />
            ) : (
              <ChevronDown className="w-3 h-3 shrink-0 text-zinc-500" />
            )}
          </button>

          {/* Picker dropdown */}
          {modelPickerOpen && (
            <div className="absolute top-full mt-1 left-0 w-72 bg-zinc-900 border border-zinc-800 rounded-xl shadow-xl shadow-black/40 z-50 animate-scale-in overflow-hidden">
              {/* Search */}
              <div className="p-2 border-b border-zinc-800">
                <div className="relative">
                  <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-zinc-500" />
                  <input
                    type="text"
                    value={modelSearch}
                    onChange={(e) => setModelSearch(e.target.value)}
                    placeholder="Search models..."
                    className="w-full bg-zinc-950 border border-zinc-800 rounded-lg pl-8 pr-3 py-1.5 text-xs text-white placeholder:text-zinc-600 focus:outline-none focus:ring-1 focus:ring-zinc-700"
                    autoFocus
                  />
                </div>
              </div>
              {/* List */}
              <div className="max-h-64 overflow-y-auto">
                {filteredModels.length === 0 ? (
                  <div className="px-3 py-4 text-xs text-zinc-500 text-center">
                    {loadingModels
                      ? "Loading models..."
                      : modelsError
                        ? "Network issue, check your API key or routing"
                        : "No models found"}
                  </div>
                ) : (
                  filteredModels.map((m) => (
                    <button
                      key={m.id}
                      onClick={() => {
                        updateSettings({ model: m.id });
                        setModelPickerOpen(false);
                        setModelSearch("");
                      }}
                      className={
                        "w-full text-left px-3 py-2 text-xs transition-colors border-b border-zinc-800/50 last:border-b-0 " +
                        (settings.model === m.id
                          ? "bg-zinc-800 text-white"
                          : "text-zinc-400 hover:bg-zinc-800/50 hover:text-zinc-200")
                      }
                    >
                      <span className="block truncate font-medium">
                        {m.label}
                      </span>
                      {m.label !== m.id && (
                        <span className="block truncate text-zinc-600 text-[10px] mt-0.5">
                          {m.id}
                        </span>
                      )}
                    </button>
                  ))
                )}
              </div>
            </div>
          )}
        </div>

        <button
          onClick={() => setShowModal(true)}
          className="p-2 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-colors btn-press"
          title="Settings"
        >
          <Settings className="w-4.5 h-4.5" />
        </button>
      </div>

      {/* Go Button */}
      <button
        onClick={handleSubmit}
        disabled={loading || !task.trim()}
        className={clsx(
          "mt-6 px-8 py-2.5 rounded-full font-medium text-sm flex items-center gap-2 transition-all btn-press",
          loading || !task.trim()
            ? "bg-zinc-700 text-zinc-500 cursor-not-allowed"
            : "bg-zinc-50 text-zinc-900 hover:bg-zinc-200 shadow-sm",
        )}
      >
        {loading ? (
          <Spinner size={16} className="text-zinc-400" />
        ) : (
          <Play className="w-4 h-4" />
        )}
        {loading ? "Starting..." : "Go"}
      </button>

      {/* Suggested Prompts */}
      <div
        className="mt-16 w-full max-w-3xl animate-slide-up"
        style={{ animationDelay: "0.15s" }}
      >
        <p className="text-zinc-500 text-sm font-medium mb-4 text-center">
          Suggested Actions
        </p>
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-3">
          {[
            {
              icon: Globe,
              title: "Web Research",
              desc: "Find the latest news on AI models.",
              prompt:
                "Open a web browser and find the latest news regarding Anthropic and OpenAI models.",
            },
            {
              icon: FileCode2,
              title: "Code Review",
              desc: "Open VSCode and debug my app.",
              prompt:
                "Open VSCode, look at my currently open file, and see if there are any obvious bugs.",
            },
            {
              icon: FileText,
              title: "Extract Data",
              desc: "Read a PDF and extract to CSV.",
              prompt:
                "Open the PDF on my desktop and extract the data table into a new CSV file.",
            },
            {
              icon: ImageIcon,
              title: "UI Testing",
              desc: "Test UI elements on this website.",
              prompt:
                "Open Chrome, go to example.com, and verify that the main button is clickable.",
            },
          ].map((item, i) => (
            <button
              key={i}
              onClick={() => {
                setTask(item.prompt);
              }}
              className="flex flex-col items-start px-4 py-3 bg-zinc-900/50 border border-zinc-800/50 hover:bg-zinc-800 hover:border-zinc-700/80 rounded-xl transition-colors text-left btn-press"
            >
              <div className="flex items-center gap-2 mb-1.5 ">
                <item.icon className="w-4 h-4 text-blue-400" />
                <span className="text-sm font-medium text-zinc-200">
                  {item.title}
                </span>
              </div>
              <p className="text-[11px] text-zinc-500 leading-snug">
                {item.desc}
              </p>
            </button>
          ))}
        </div>
      </div>

      {/* Settings Modal */}
      {showModal && <SettingsModal />}
    </div>
  );
}
