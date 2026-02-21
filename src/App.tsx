import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { useUIStore } from "./stores/uiStore";
import { useAgentStore } from "./stores/agentStore";
import { useSettingsStore } from "./stores/settingsStore";
import { EVENTS } from "./lib/events";
import { restoreDefaultWindow } from "./lib/windowManager";
import { StartScreen } from "./components/StartScreen";
import { Taskbar } from "./components/Taskbar";
import { TaskbarExpanded } from "./components/TaskbarExpanded";
import { SetupWizard } from "./components/SetupWizard";
import { ToastContainer } from "./components/Toast";
import { type } from "@tauri-apps/plugin-os";
import "./index.css";

function App() {
  const mode = useUIStore((s) => s.mode);
  const addToast = useUIStore((s) => s.addToast);
  const expanded = useUIStore((s) => s.expanded);
  const setMode = useUIStore((s) => s.setMode);
  const { settings, loaded, updateSettings, saveSettings } = useSettingsStore();
  const {
    setStatus,
    addAction,
    setThinking,
    setMessage,
    setEstimatedSeconds,
    setScreenshot,
    setCursor,
    reset,
  } = useAgentStore();
  const { loadSettings } = useSettingsStore();

  const [disclaimerAccepted, setDisclaimerAccepted] = useState(false);

  // Load settings on mount
  useEffect(() => {
    loadSettings();
  }, []);

  // Check for Linux and Warn
  useEffect(() => {
    async function checkOs() {
      if (type() === "linux") {
        addToast(
          "Accessibility is not supported on Linux. Experience will be degraded.",
          "error",
        );
      }
    }
    checkOs();
  }, []);

  // Listen to backend events
  useEffect(() => {
    const unlisten: (() => void)[] = [];

    listen<string>(EVENTS.AGENT_STATUS_CHANGED, (e) => {
      const payload = e.payload;
      if (payload === "idle") {
        setStatus("idle");
        restoreDefaultWindow();
        setMode("start");
        reset();
      } else if (payload === "running") {
        setStatus("running");
      } else if (payload.startsWith("error:")) {
        setStatus("error");
        const errMsg = payload.slice(6);
        setMessage(errMsg);
        addToast(`Agent Error: ${errMsg}`, "error");
        setMode("start");
        restoreDefaultWindow();
      }
    }).then((u) => unlisten.push(u));

    listen<{ base64: string; w: number; h: number }>(
      EVENTS.SCREENSHOT_UPDATED,
      (e) => {
        setScreenshot(e.payload.base64);
      },
    ).then((u) => unlisten.push(u));

    listen<{
      action_type: string;
      description: string;
      timestamp: string;
      iteration: number;
    }>(EVENTS.ACTION_EXECUTED, (e) => {
      addAction({
        id: crypto.randomUUID(),
        ...e.payload,
      });
      setThinking("");
    }).then((u) => unlisten.push(u));

    listen<{ text: string }>(EVENTS.AGENT_THINKING, (e) => {
      setThinking(e.payload.text);
    }).then((u) => unlisten.push(u));

    listen<{ text: string }>(EVENTS.AGENT_MESSAGE, (e) => {
      setMessage(e.payload.text);
    }).then((u) => unlisten.push(u));

    listen<{ x: number; y: number }>(EVENTS.CURSOR_MOVED, (e) => {
      setCursor(e.payload.x, e.payload.y);
    }).then((u) => unlisten.push(u));

    listen<{ seconds_remaining: number }>(EVENTS.ESTIMATED_TIME, (e) => {
      setEstimatedSeconds(e.payload.seconds_remaining);
    }).then((u) => unlisten.push(u));

    return () => {
      unlisten.forEach((u) => u());
    };
  }, []);

  // Show nothing until settings are loaded
  if (!loaded) return null;

  // Setup wizard gate
  if (!settings.setup_complete) {
    return (
      <>
        <SetupWizard
          onComplete={() => {
            updateSettings({ setup_complete: true });
            saveSettings();
          }}
        />
        <ToastContainer />
      </>
    );
  }

  // Disclaimer gate
  if (!disclaimerAccepted) {
    return (
      <div className="flex flex-col items-center justify-center p-8 h-full start-screen-bg text-center">
        <div className="max-w-md w-full p-6 card-surface rounded-2xl flex flex-col items-center gap-4">
          <div className="w-12 h-12 rounded-full bg-red-500/20 flex items-center justify-center text-red-400 mb-2">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" />
              <path d="M12 9v4" />
              <path d="M12 17h.01" />
            </svg>
          </div>
          <h2 className="text-xl font-semibold text-zinc-100">Disclaimer</h2>
          <p className="text-sm text-zinc-400 leading-relaxed text-left border-l-2 border-red-500/50 pl-3">
            Delegant will give an AI agent{" "}
            <strong className="text-zinc-200">
              full access to your computer
            </strong>
            . It can click, type, access files, and navigate the web on your
            behalf.
          </p>
          <p className="text-sm text-zinc-400 leading-relaxed text-left border-l-2 border-red-500/50 pl-3">
            In the worst-case scenario where the agent is misbehaving or stuck,
            press <strong className="text-zinc-200">CTRL+ALT+DEL</strong> or
            close the application immediately to suspend the agent.
          </p>
          <button
            onClick={() => {
              setDisclaimerAccepted(true);
            }}
            className="mt-4 w-full px-4 py-2.5 bg-blue-600 hover:bg-blue-500 transition-colors text-white font-medium rounded-lg shadow-md"
          >
            I Agree, Continue
          </button>
        </div>
      </div>
    );
  }

  if (mode === "start") {
    return (
      <div key="start" className="h-full animate-fade-in">
        <StartScreen />
        <ToastContainer />
      </div>
    );
  }

  // Taskbar mode
  return (
    <div key="taskbar" className="h-full animate-fade-in">
      {expanded ? <TaskbarExpanded /> : <Taskbar />}
      <ToastContainer />
    </div>
  );
}

export default App;
