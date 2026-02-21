export type UIMode = "start" | "taskbar";

export type AgentStatus = "idle" | "running" | "paused" | "error";

export interface Settings {
  anthropic_api_key: string;
  openai_api_key: string;
  openrouter_api_key: string;
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

export const PROVIDERS = [
  { id: "anthropic", label: "Anthropic" },
  { id: "openai", label: "OpenAI" },
  { id: "openrouter", label: "OpenRouter" },
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
};

/** Fetch models dynamically for any provider. Falls back to static list. */
export async function fetchModels(
  provider: string,
  apiKey: string
): Promise<{ id: string; label: string }[]> {
  if (!apiKey) return MODELS[provider] ?? [];

  try {
    if (provider === "anthropic") {
      return await fetchAnthropicModels(apiKey);
    } else if (provider === "openai") {
      return await fetchOpenAIModels(apiKey);
    } else if (provider === "openrouter") {
      return await fetchOpenRouterModels(apiKey);
    }
  } catch {
    // Fall back to static
  }
  return MODELS[provider] ?? [];
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
  if (!res.ok) return MODELS.anthropic;
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
  return models.length > 0 ? models : MODELS.anthropic;
}

async function fetchOpenAIModels(
  apiKey: string
): Promise<{ id: string; label: string }[]> {
  const res = await fetch("https://api.openai.com/v1/models", {
    headers: { Authorization: `Bearer ${apiKey}` },
  });
  if (!res.ok) return MODELS.openai;
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
  return models.length > 0 ? models : MODELS.openai;
}

async function fetchOpenRouterModels(
  apiKey: string
): Promise<{ id: string; label: string }[]> {
  const res = await fetch("https://openrouter.ai/api/v1/models", {
    headers: apiKey ? { Authorization: `Bearer ${apiKey}` } : {},
  });
  if (!res.ok) return MODELS.openrouter;
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
  return models.length > 0 ? models : MODELS.openrouter;
}
