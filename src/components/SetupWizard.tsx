import { useState, useEffect } from "react";
import {
  Sparkles,
  CheckCircle,
  ChevronRight,
  ChevronLeft,
  Eye,
  EyeOff,
  Monitor,
} from "lucide-react";
import { clsx } from "clsx";
import { useSettingsStore } from "../stores/settingsStore";
import { PROVIDERS, MODELS, fetchModelsDetailed } from "../lib/types";
import { Spinner } from "./Spinner";

const STEPS = ["Welcome", "Provider", "API Key", "Display", "Ready"] as const;

const providerDescriptions: Record<string, string> = {
  anthropic: "Claude models with native computer use",
  openai: "GPT-4.1 and o-series with vision",
  openrouter: "Access many models via one API key",
  ollama: "Run local models via Ollama",
};

export function SetupWizard({ onComplete }: { onComplete: () => void }) {
  const { settings, updateSettings, saveSettings } = useSettingsStore();
  const [step, setStep] = useState(0);
  const [direction, setDirection] = useState<"left" | "right">("left");
  const [showKey, setShowKey] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<"success" | "error" | null>(
    null,
  );

  // Auto-detect resolution
  useEffect(() => {
    (async () => {
      try {
        const { currentMonitor } = await import("@tauri-apps/api/window");
        const monitor = await currentMonitor();
        if (monitor) {
          updateSettings({
            display_width: monitor.size.width,
            display_height: monitor.size.height,
          });
        }
      } catch {
        // Ignore
      }
    })();
  }, []);

  const goNext = () => {
    if (step < STEPS.length - 1) {
      setDirection("left");
      setStep(step + 1);
    }
  };

  const goBack = () => {
    if (step > 0) {
      setDirection("right");
      setStep(step - 1);
    }
  };

  const getApiKeyForProvider = () => {
    if (settings.provider === "anthropic") return settings.anthropic_api_key;
    if (settings.provider === "openrouter") return settings.openrouter_api_key;
    if (settings.provider === "ollama") return settings.ollama_api_key;
    return settings.openai_api_key;
  };

  const setApiKeyForProvider = (value: string) => {
    if (settings.provider === "anthropic")
      updateSettings({ anthropic_api_key: value });
    else if (settings.provider === "openrouter")
      updateSettings({ openrouter_api_key: value });
    else if (settings.provider === "ollama")
      updateSettings({ ollama_api_key: value });
    else updateSettings({ openai_api_key: value });
  };

  const testConnection = async () => {
    setTesting(true);
    setTestResult(null);

    const key = getApiKeyForProvider();
    try {
      const result = await fetchModelsDetailed(
        settings.provider,
        key,
        settings.ollama_base_url,
      );
      if (!result.hadError && result.models.length > 0) {
        setTestResult("success");
      } else {
        setTestResult("error");
      }
    } catch {
      setTestResult("error");
    }

    setTesting(false);
  };

  const canProceed = () => {
    if (step === 2) {
      if (settings.provider === "ollama") return true;
      return getApiKeyForProvider().length > 10;
    }
    if (step === 3)
      return settings.display_width > 0 && settings.display_height > 0;
    return true;
  };

  const handleComplete = async () => {
    updateSettings({ setup_complete: true });
    await saveSettings();
    onComplete();
  };

  const animationClass =
    direction === "left" ? "animate-slide-left" : "animate-slide-right";

  return (
    <div className="flex flex-col items-center justify-center h-full bg-zinc-950 px-8 select-none">
      {/* Progress dots */}
      <div className="flex items-center gap-2 mb-8">
        {STEPS.map((_, i) => (
          <div
            key={i}
            className={clsx(
              "h-1.5 rounded-full transition-all duration-300",
              i === step
                ? "w-6 bg-blue-500"
                : i < step
                  ? "w-1.5 bg-blue-500/50"
                  : "w-1.5 bg-zinc-700",
            )}
          />
        ))}
      </div>

      {/* Step content */}
      <div key={step} className={clsx("w-full max-w-md", animationClass)}>
        {step === 0 && (
          <div className="flex flex-col items-center text-center">
            <Sparkles className="w-12 h-12 text-blue-400 mb-4" />
            <h1 className="text-2xl font-semibold text-white mb-2">
              Welcome to Delegant
            </h1>
            <p className="text-zinc-400 text-sm leading-relaxed max-w-sm">
              An AI agent that can see your screen and control your computer to
              accomplish tasks. Let's get you set up in a few quick steps.
            </p>
          </div>
        )}

        {step === 1 && (
          <div className="flex flex-col items-center">
            <h2 className="text-xl font-semibold text-white mb-1">
              Choose a Provider
            </h2>
            <p className="text-zinc-500 text-sm mb-6">
              Select the AI provider you'd like to use
            </p>
            <div className="grid gap-3 w-full">
              {PROVIDERS.map((p) => (
                <button
                  key={p.id}
                  onClick={() => {
                    updateSettings({
                      provider: p.id,
                      model: MODELS[p.id]?.[0]?.id ?? "",
                    });
                  }}
                  className={clsx(
                    "flex items-center gap-3 px-4 py-3 rounded-xl border text-left transition-all btn-press",
                    settings.provider === p.id
                      ? "bg-blue-600/15 border-blue-500/40 ring-1 ring-blue-500/20"
                      : "bg-zinc-900 border-zinc-800 hover:bg-zinc-800",
                  )}
                >
                  <div className="flex-1">
                    <p
                      className={clsx(
                        "text-sm font-medium",
                        settings.provider === p.id
                          ? "text-blue-300"
                          : "text-zinc-300",
                      )}
                    >
                      {p.label}
                    </p>
                    <p className="text-xs text-zinc-500 mt-0.5">
                      {providerDescriptions[p.id]}
                    </p>
                  </div>
                  {settings.provider === p.id && (
                    <CheckCircle className="w-5 h-5 text-blue-400 shrink-0" />
                  )}
                </button>
              ))}
            </div>
          </div>
        )}

        {step === 2 && (
          <div className="flex flex-col items-center">
            <h2 className="text-xl font-semibold text-white mb-1">
              {settings.provider === "ollama"
                ? "Connect to Ollama"
                : "Enter API Key"}
            </h2>
            <p className="text-zinc-500 text-sm mb-6">
              {settings.provider === "ollama"
                ? "Use a local Ollama server (key optional)"
                : `Your ${PROVIDERS.find((p) => p.id === settings.provider)?.label} API key`}
            </p>
            <div className="w-full space-y-4">
              {settings.provider === "ollama" && (
                <input
                  type="text"
                  value={settings.ollama_base_url}
                  onChange={(e) =>
                    updateSettings({ ollama_base_url: e.target.value })
                  }
                  placeholder="http://127.0.0.1:11434"
                  className="w-full bg-zinc-950 border border-zinc-800 rounded-xl px-4 py-3 text-sm text-white placeholder:text-zinc-600 focus:outline-none focus:ring-2 focus:ring-blue-500/40 input-glow"
                />
              )}
              <div className="relative">
                <input
                  type={showKey ? "text" : "password"}
                  value={getApiKeyForProvider()}
                  onChange={(e) => setApiKeyForProvider(e.target.value)}
                  placeholder={
                    settings.provider === "anthropic"
                      ? "sk-ant-..."
                      : settings.provider === "openrouter"
                        ? "sk-or-..."
                        : settings.provider === "ollama"
                          ? "Optional (leave empty for local Ollama)"
                        : "sk-..."
                  }
                  className="w-full bg-zinc-950 border border-zinc-800 rounded-xl px-4 py-3 text-sm text-white placeholder:text-zinc-600 focus:outline-none focus:ring-2 focus:ring-blue-500/40 pr-10 input-glow"
                  autoFocus
                />
                <button
                  type="button"
                  onClick={() => setShowKey(!showKey)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-zinc-500 hover:text-zinc-300"
                >
                  {showKey ? (
                    <EyeOff className="w-4 h-4" />
                  ) : (
                    <Eye className="w-4 h-4" />
                  )}
                </button>
              </div>
              <button
                onClick={testConnection}
                disabled={
                  testing ||
                  (!getApiKeyForProvider() && settings.provider !== "ollama")
                }
                className={clsx(
                  "w-full py-2.5 rounded-xl text-sm font-medium transition-all btn-press",
                  testing ||
                    (!getApiKeyForProvider() && settings.provider !== "ollama")
                    ? "bg-zinc-800 text-zinc-500 cursor-not-allowed"
                    : "bg-zinc-800 text-zinc-300 hover:bg-zinc-700",
                )}
              >
                {testing ? (
                  <span className="flex items-center justify-center gap-2">
                    <Spinner size={14} className="text-zinc-400" />
                    Testing...
                  </span>
                ) : (
                  "Test Connection"
                )}
              </button>
              {testResult === "success" && (
                <p className="text-green-400 text-sm text-center flex items-center justify-center gap-1.5">
                  <CheckCircle className="w-4 h-4" />
                  {settings.provider === "ollama"
                    ? "Connection works"
                    : "Key looks valid"}
                </p>
              )}
              {testResult === "error" && (
                <p className="text-red-400 text-sm text-center">
                  Network issue, check your API key or routing
                </p>
              )}
            </div>
          </div>
        )}

        {step === 3 && (
          <div className="flex flex-col items-center">
            <Monitor className="w-10 h-10 text-blue-400 mb-3" />
            <h2 className="text-xl font-semibold text-white mb-1">
              Display Settings
            </h2>
            <p className="text-zinc-500 text-sm mb-6">
              Resolution the AI will see (auto-detected)
            </p>
            <div className="grid grid-cols-2 gap-3 w-full">
              <div>
                <label className="block text-sm text-zinc-400 mb-1.5">
                  Width
                </label>
                <input
                  type="number"
                  value={settings.display_width}
                  onChange={(e) =>
                    updateSettings({ display_width: Number(e.target.value) })
                  }
                  className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-sm text-zinc-300 focus:outline-none focus:ring-2 focus:ring-blue-500/40 input-glow"
                />
              </div>
              <div>
                <label className="block text-sm text-zinc-400 mb-1.5">
                  Height
                </label>
                <input
                  type="number"
                  value={settings.display_height}
                  onChange={(e) =>
                    updateSettings({ display_height: Number(e.target.value) })
                  }
                  className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-sm text-zinc-300 focus:outline-none focus:ring-2 focus:ring-blue-500/40 input-glow"
                />
              </div>
            </div>
          </div>
        )}

        {step === 4 && (
          <div className="flex flex-col items-center text-center">
            <div className="w-14 h-14 rounded-full bg-green-500/15 flex items-center justify-center mb-4">
              <CheckCircle className="w-8 h-8 text-green-400" />
            </div>
            <h2 className="text-xl font-semibold text-white mb-2">
              You're all set!
            </h2>
            <div className="text-zinc-500 text-sm space-y-1 mb-2">
              <p>
                Provider:{" "}
                <span className="text-zinc-300">
                  {PROVIDERS.find((p) => p.id === settings.provider)?.label}
                </span>
              </p>
              <p>
                Model:{" "}
                <span className="text-zinc-300">
                  {MODELS[settings.provider]?.find(
                    (m) => m.id === settings.model,
                  )?.label ?? settings.model}
                </span>
              </p>
              <p>
                Resolution:{" "}
                <span className="text-zinc-300">
                  {settings.display_width} x {settings.display_height}
                </span>
              </p>
            </div>
          </div>
        )}
      </div>

      {/* Navigation */}
      <div className="flex items-center gap-3 mt-8">
        {step > 0 && (
          <button
            onClick={goBack}
            className="px-5 py-2 rounded-full text-sm text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-all flex items-center gap-1 btn-press"
          >
            <ChevronLeft className="w-4 h-4" />
            Back
          </button>
        )}

        {step < STEPS.length - 1 ? (
          <button
            onClick={goNext}
            disabled={!canProceed()}
            className={clsx(
              "px-6 py-2 rounded-full text-sm font-medium transition-all flex items-center gap-1 btn-press",
              canProceed()
                ? "bg-zinc-50 text-zinc-900 hover:bg-zinc-200 shadow-sm"
                : "bg-zinc-800 text-zinc-500 cursor-not-allowed",
            )}
          >
            {step === 0 ? "Get Started" : "Next"}
            <ChevronRight className="w-4 h-4" />
          </button>
        ) : (
          <button
            onClick={handleComplete}
            className="px-8 py-2.5 rounded-full text-sm font-medium bg-zinc-50 text-zinc-900 hover:bg-zinc-200 transition-all shadow-sm btn-press"
          >
            Start Using
          </button>
        )}

        {step > 0 && step < STEPS.length - 1 && step !== 2 && (
          <button
            onClick={goNext}
            className="px-4 py-2 rounded-full text-xs text-zinc-600 hover:text-zinc-400 transition-colors"
          >
            Skip
          </button>
        )}
      </div>
    </div>
  );
}
