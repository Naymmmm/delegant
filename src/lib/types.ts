import { commands } from "./commands";

export type UIMode = "start" | "taskbar";

export type AgentStatus = "idle" | "running" | "paused" | "error";

export interface Settings {
  anthropic_api_key: string;
  openai_api_key: string;
  openrouter_api_key: string;
  ollama_api_key: string;
  ollama_base_url: string;
  provider: string;
  model: string;
  display_width: number;
  display_height: number;
  shell_timeout_secs: number;
  setup_complete: boolean;
}

export interface ActionEntry {
  id: string;
  action_type: string;
  description: string;
  timestamp: string;
  iteration: number;
  output?: string;
}

export interface CaptureResult {
  base64: string;
  orig_width: number;
  orig_height: number;
  scaled_width: number;
  scaled_height: number;
  scale_factor: number;
}

export interface ShellResult {
  stdout: string;
  stderr: string;
  exit_code: number;
}

export interface WindowInfo {
  handle: number;
  title: string;
}

export interface ModelFetchResult {
  models: { id: string; label: string }[];
  hadError: boolean;
}

export const PROVIDERS = [
  { id: "anthropic", label: "Anthropic" },
  { id: "openai", label: "OpenAI" },
  { id: "openrouter", label: "OpenRouter" },
  { id: "ollama", label: "Ollama" },
] as const;

/** Fallback static model lists */
export const MODELS: Record<string, { id: string; label: string }[]> = {
  anthropic: [
    { id: "claude-sonnet-4-6", label: "Claude Sonnet 4.6" },
    { id: "claude-opus-4-6", label: "Claude Opus 4.6" },
    { id: "claude-haiku-4-5", label: "Claude Haiku 4.5" },
    { id: "claude-sonnet-4-5", label: "Claude Sonnet 4.5" },
    { id: "claude-opus-4-5", label: "Claude Opus 4.5" },
  ],
  openai: [
    { id: "gpt-4.1", label: "GPT-4.1" },
    { id: "gpt-4.1-mini", label: "GPT-4.1 Mini" },
    { id: "gpt-4.1-nano", label: "GPT-4.1 Nano" },
    { id: "o3", label: "o3" },
    { id: "o4-mini", label: "o4-mini" },
  ],
  openrouter: [
    { id: "anthropic/claude-sonnet-4.6", label: "Claude Sonnet 4.6" },
    { id: "anthropic/claude-opus-4.6", label: "Claude Opus 4.6" },
    { id: "openai/gpt-4.1", label: "GPT-4.1" },
    { id: "openai/gpt-4.1-mini", label: "GPT-4.1 Mini" },
    { id: "google/gemini-2.5-pro", label: "Gemini 2.5 Pro" },
    { id: "google/gemini-2.5-flash", label: "Gemini 2.5 Flash" },
    { id: "meta-llama/llama-4-maverick", label: "Llama 4 Maverick" },
  ],
  ollama: [
    { id: "llama3.2:latest", label: "Llama 3.2" },
    { id: "qwen2.5:latest", label: "Qwen 2.5" },
    { id: "phi4:latest", label: "Phi-4" },
  ],
};

/** Fetch models dynamically for any provider. */
export async function fetchModels(
  provider: string,
  apiKey: string,
  ollamaBaseUrl?: string
): Promise<{ id: string; label: string }[]> {
  const result = await fetchModelsDetailed(provider, apiKey, ollamaBaseUrl);
  return result.models;
}

export async function fetchModelsDetailed(
  provider: string,
  apiKey: string,
  ollamaBaseUrl?: string
): Promise<ModelFetchResult> {
  if (!apiKey && provider !== "ollama") {
    return { models: [], hadError: true };
  }

  try {
    if (provider === "anthropic") {
      return { models: await fetchAnthropicModels(apiKey), hadError: false };
    } else if (provider === "openai") {
      return { models: await fetchOpenAIModels(apiKey), hadError: false };
    } else if (provider === "openrouter") {
      return { models: await fetchOpenRouterModels(apiKey), hadError: false };
    } else if (provider === "ollama") {
      return {
        models: await fetchOllamaModels(ollamaBaseUrl, apiKey),
        hadError: false,
      };
    }
  } catch {
    return { models: [], hadError: true };
  }
  return { models: [], hadError: true };
}

async function fetchAnthropicModels(
  apiKey: string
): Promise<{ id: string; label: string }[]> {
  const res = await fetch("https://api.anthropic.com/v1/models", {
    headers: {
      "x-api-key": apiKey,
      "anthropic-version": "2023-06-01",
    },
  });
  if (!res.ok) throw new Error(`Anthropic model fetch failed: ${res.status}`);
  const data = await res.json();
  const models = (data.data ?? [])
    .sort(
      (a: { created_at?: string }, b: { created_at?: string }) =>
        (b.created_at ?? "").localeCompare(a.created_at ?? "")
    )
    .map((m: { id: string; display_name?: string }) => ({
      id: m.id,
      label: m.display_name ?? m.id,
    }));
  return models;
}

async function fetchOpenAIModels(
  apiKey: string
): Promise<{ id: string; label: string }[]> {
  const res = await fetch("https://api.openai.com/v1/models", {
    headers: { Authorization: `Bearer ${apiKey}` },
  });
  if (!res.ok) throw new Error(`OpenAI model fetch failed: ${res.status}`);
  const data = await res.json();
  const models = (data.data ?? [])
    .sort(
      (a: { created?: number }, b: { created?: number }) =>
        (b.created ?? 0) - (a.created ?? 0)
    )
    .map((m: { id: string }) => ({
      id: m.id,
      label: m.id,
    }));
  return models;
}

async function fetchOpenRouterModels(
  apiKey: string
): Promise<{ id: string; label: string }[]> {
  const res = await fetch("https://openrouter.ai/api/v1/models", {
    headers: apiKey ? { Authorization: `Bearer ${apiKey}` } : {},
  });
  if (!res.ok) throw new Error(`OpenRouter model fetch failed: ${res.status}`);
  const data = await res.json();
  const models = (data.data ?? [])
    .sort(
      (a: { created?: number }, b: { created?: number }) =>
        (b.created ?? 0) - (a.created ?? 0)
    )
    .map((m: { id: string; name?: string }) => ({
      id: m.id,
      label: m.name ?? m.id,
    }));
  return models;
}

async function fetchOllamaModels(
  baseUrl?: string,
  apiKey?: string
): Promise<{ id: string; label: string }[]> {
  return commands.listOllamaModels(
    normalizeOllamaBaseUrl(baseUrl),
    apiKey?.trim() || undefined,
  );
}

function normalizeOllamaBaseUrl(baseUrl?: string): string {
  const raw = (baseUrl ?? "").trim();
  let url =
    raw.length === 0
      ? "http://127.0.0.1:11434"
      : raw.startsWith("http://") || raw.startsWith("https://")
        ? raw
        : `http://${raw}`;

  url = url.replace(/\/+$/, "");
  if (url.endsWith("/v1")) {
    url = url.slice(0, -3);
  }
  url = url.replace(/\/+$/, "");

  return url;
}
