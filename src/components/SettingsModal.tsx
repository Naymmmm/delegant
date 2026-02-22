import { useState, useEffect, useMemo } from "react";
import { X, Eye, EyeOff, RotateCcw, Search, Loader2 } from "lucide-react";
import { useSettingsStore } from "../stores/settingsStore";
import { PROVIDERS, MODELS, fetchModelsDetailed } from "../lib/types";

export function SettingsModal() {
  const { settings, updateSettings, saveSettings, setShowModal, resetWizard } =
    useSettingsStore();
  const [showKey, setShowKey] = useState(false);
  const [dynamicModels, setDynamicModels] = useState(
    MODELS[settings.provider] ?? []
  );
  const [loadingModels, setLoadingModels] = useState(false);
  const [modelSearch, setModelSearch] = useState("");
  const [modelsError, setModelsError] = useState(false);

  // Fetch models dynamically when provider or key changes
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
    setModelSearch("");
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

  const handleClose = async () => {
    await saveSettings();
    setShowModal(false);
  };

  const getApiKey = () => {
    if (settings.provider === "anthropic") return settings.anthropic_api_key;
    if (settings.provider === "openrouter") return settings.openrouter_api_key;
    if (settings.provider === "ollama") return settings.ollama_api_key;
    return settings.openai_api_key;
  };

  const setApiKey = (value: string) => {
    if (settings.provider === "anthropic")
      updateSettings({ anthropic_api_key: value });
    else if (settings.provider === "openrouter")
      updateSettings({ openrouter_api_key: value });
    else if (settings.provider === "ollama")
      updateSettings({ ollama_api_key: value });
    else updateSettings({ openai_api_key: value });
  };

  const keyPlaceholder =
    settings.provider === "anthropic"
      ? "sk-ant-..."
      : settings.provider === "openrouter"
        ? "sk-or-..."
        : settings.provider === "ollama"
          ? "Optional (leave empty for local Ollama)"
        : "sk-...";

  const filteredModels = useMemo(() => {
    if (!modelSearch.trim()) return dynamicModels;
    const q = modelSearch.toLowerCase();
    return dynamicModels.filter(
      (m) =>
        m.id.toLowerCase().includes(q) || m.label.toLowerCase().includes(q)
    );
  }, [dynamicModels, modelSearch]);

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
      <div className="bg-zinc-900 border border-zinc-800 rounded-2xl w-full max-w-md shadow-xl shadow-black/40 animate-scale-in overflow-hidden flex flex-col max-h-[85vh]">
        {/* Header */}
        <div className="flex items-center justify-between px-5 pt-5 pb-3 shrink-0">
          <h2 className="text-base font-semibold text-white">Settings</h2>
          <button
            onClick={handleClose}
            className="p-1.5 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="px-5 pb-5 space-y-4 overflow-y-auto flex-1 min-h-0">
          {/* Provider */}
          <div>
            <label className="block text-xs font-medium text-zinc-400 mb-1.5">
              Provider
            </label>
            <div className="flex bg-zinc-950 rounded-lg p-0.5 border border-zinc-800">
              {PROVIDERS.map((p) => (
                <button
                  key={p.id}
                  onClick={() => {
                    updateSettings({
                      provider: p.id,
                      model: (MODELS[p.id] ?? [])[0]?.id ?? "",
                    });
                  }}
                  className={
                    "flex-1 px-3 py-1.5 rounded-md text-xs font-medium transition-all " +
                    (settings.provider === p.id
                      ? "bg-zinc-800 text-white shadow-sm"
                      : "text-zinc-500 hover:text-zinc-300")
                  }
                >
                  {p.label}
                </button>
              ))}
            </div>
          </div>

          {/* API Key */}
          <div>
            <label className="block text-xs font-medium text-zinc-400 mb-1.5">
              {PROVIDERS.find((p) => p.id === settings.provider)?.label} API Key
              {settings.provider === "ollama" ? " (Optional)" : ""}
            </label>
            <div className="relative">
              <input
                type={showKey ? "text" : "password"}
                value={getApiKey()}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder={keyPlaceholder}
                className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-sm text-white placeholder:text-zinc-600 focus:outline-none focus:ring-1 focus:ring-zinc-700 pr-10"
              />
              <button
                type="button"
                onClick={() => setShowKey(!showKey)}
                className="absolute right-2.5 top-1/2 -translate-y-1/2 text-zinc-500 hover:text-zinc-300"
              >
                {showKey ? (
                  <EyeOff className="w-3.5 h-3.5" />
                ) : (
                  <Eye className="w-3.5 h-3.5" />
                )}
              </button>
            </div>
          </div>

          {settings.provider === "ollama" && (
            <div>
              <label className="block text-xs font-medium text-zinc-400 mb-1.5">
                Ollama URL
              </label>
              <input
                type="text"
                value={settings.ollama_base_url}
                onChange={(e) =>
                  updateSettings({ ollama_base_url: e.target.value })
                }
                placeholder="http://127.0.0.1:11434"
                className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-sm text-white placeholder:text-zinc-600 focus:outline-none focus:ring-1 focus:ring-zinc-700"
              />
            </div>
          )}

          {/* Model — scrollable list */}
          <div>
            <label className="block text-xs font-medium text-zinc-400 mb-1.5">
              Model
              {loadingModels && (
                <Loader2 className="w-3 h-3 inline ml-1.5 animate-spinner text-zinc-500" />
              )}
            </label>
            {/* Search */}
            <div className="relative mb-2">
              <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-zinc-500" />
              <input
                type="text"
                value={modelSearch}
                onChange={(e) => setModelSearch(e.target.value)}
                placeholder="Search models..."
                className="w-full bg-zinc-950 border border-zinc-800 rounded-lg pl-8 pr-3 py-1.5 text-xs text-white placeholder:text-zinc-600 focus:outline-none focus:ring-1 focus:ring-zinc-700"
              />
            </div>
            {/* List */}
            <div className="bg-zinc-950 border border-zinc-800 rounded-lg max-h-48 overflow-y-auto">
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
                    onClick={() => updateSettings({ model: m.id })}
                    className={
                      "w-full text-left px-3 py-2 text-xs transition-colors border-b border-zinc-800/50 last:border-b-0 " +
                      (settings.model === m.id
                        ? "bg-zinc-800 text-white"
                        : "text-zinc-400 hover:bg-zinc-900 hover:text-zinc-200")
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

          {/* Display Resolution */}
          <div>
            <label className="block text-xs font-medium text-zinc-400 mb-1.5">
              Display Resolution
            </label>
            <div className="grid grid-cols-2 gap-2">
              <input
                type="number"
                value={settings.display_width}
                onChange={(e) =>
                  updateSettings({ display_width: Number(e.target.value) })
                }
                className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-sm text-zinc-300 focus:outline-none focus:ring-1 focus:ring-zinc-700"
                placeholder="Width"
              />
              <input
                type="number"
                value={settings.display_height}
                onChange={(e) =>
                  updateSettings({ display_height: Number(e.target.value) })
                }
                className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-sm text-zinc-300 focus:outline-none focus:ring-1 focus:ring-zinc-700"
                placeholder="Height"
              />
            </div>
          </div>

          {/* Divider */}
          <div className="border-t border-zinc-800" />

          {/* Reset Wizard */}
          <button
            onClick={resetWizard}
            className="flex items-center gap-2 text-xs text-zinc-500 hover:text-zinc-300 transition-colors"
          >
            <RotateCcw className="w-3.5 h-3.5" />
            Reset setup wizard
          </button>
        </div>
      </div>
    </div>
  );
}
