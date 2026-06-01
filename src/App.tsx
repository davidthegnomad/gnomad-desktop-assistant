import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  Settings as SettingsIcon,
  BookOpen,
  Clipboard,
  ShieldAlert,
  Cpu,
  RefreshCw,
  AlertCircle,
  X,
  ChevronRight,
  Eye,
  EyeOff,
  Sun,
  Moon,
  Monitor,
  Terminal,
  Sparkles,
} from "lucide-react";
import { AboutModal } from "./components/AboutModal";
import { GnomadLogo } from "./components/GnomadLogo";
import { ChatSidebar } from "./components/ChatSidebar";
import { KnowledgePanel } from "./components/KnowledgePanel";
import { OnboardingModal } from "./components/OnboardingModal";
import { WindowModeBar } from "./components/WindowModeBar";
import { useTheme } from "./hooks/useTheme";
import {
  appendUserPreference,
  getAgentContextBundle,
  trimContextBundle,
} from "./lib/knowledge";
import { getStoredSidebarSide, type SidebarSide } from "./lib/sidebarLayout";
import {
  createChatSession,
  deleteChatSession,
  getChatStore,
  loadChatSession,
  saveChatSession,
  setActiveChatSession,
  type ChatSessionSummary,
} from "./lib/chatHistory";
import {
  getWindowMode,
  subscribeWindowMode,
  type WindowDisplayMode,
} from "./lib/windowMode";
import {
  APP_NAME,
  ABOUT_CREDIT,
  MUSHROOM,
  STUDIO_URL,
  VERSION,
} from "./lib/brand";
import {
  getOnboardingComplete,
  getStoredLocalModel,
  getStoredModel,
  getStoredProvider,
  hasProviderConfigured,
  loadCredential,
  saveCredential,
  setStoredLocalModel,
  setStoredModel,
  setStoredProvider,
  type ProviderMode,
} from "./lib/preferences";
import { getEnvLlmConfig } from "./lib/envConfig";
import { chatCompletion } from "./lib/llm";
import {
  normalizeCloudModel,
  resolveLlmAvailability,
  type LlmAvailability,
  type ModelOption,
} from "./lib/models";

interface Message {
  role: "user" | "assistant";
  text: string;
  commandExecuted?: string;
  commandResult?: {
    success: boolean;
    stdout: string;
    stderr: string;
  };
}

interface SafetyCheck {
  is_safe: boolean;
  requires_hitl_approval: boolean;
  requires_admin: boolean;
  danger_reason: string | null;
}

const WELCOME_ONLY =
  "Hey — I'm Gnomad 🦙 I watch your active window and clipboard, run safe shell commands, and help you automate. What's on your mind?";

const SUGGESTION_PROMPTS = [
  "Summarize what I'm working on",
  "Help me automate a small task",
  "Explain this in simpler terms",
] as const;

function App() {
  const { themeMode, resolved, cycleTheme } = useTheme();
  const sudoResolveRef = useRef<((approved: boolean) => void) | null>(null);

  const [messages, setMessages] = useState<Message[]>([
    { role: "assistant", text: WELCOME_ONLY },
  ]);
  const [input, setInput] = useState("");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [knowledgeOpen, setKnowledgeOpen] = useState(false);
  const [aboutOpen, setAboutOpen] = useState(false);
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [chatSessions, setChatSessions] = useState<ChatSessionSummary[]>([]);
  const [railSide, setRailSide] = useState<SidebarSide>(getStoredSidebarSide);
  const [showOnboarding, setShowOnboarding] = useState(false);
  const [prefsLoaded, setPrefsLoaded] = useState(false);

  const [activeApp, setActiveApp] = useState("Loading...");
  const [activeTitle, setActiveTitle] = useState("Loading...");
  const [clipboardSnippet, setClipboardSnippet] = useState("Empty");

  const [apiType, setApiType] = useState<ProviderMode>("cloud");
  const [selectedModel, setSelectedModel] = useState("deepseek-chat");
  const [llmAvailability, setLlmAvailability] = useState<LlmAvailability>({
    cloudConfigured: false,
    localConfigured: false,
    cloudModels: [],
    localModels: [],
  });
  const [localModel, setLocalModel] = useState("llama3.2");
  const [ollamaUrl, setOllamaUrl] = useState("http://localhost:11434");
  const [apiKeyVisible, setApiKeyVisible] = useState(false);
  const [apiKey, setApiKey] = useState("");
  const [apiKeySource, setApiKeySource] = useState<"env" | "keychain" | "none">("none");

  const [isThinking, setIsThinking] = useState(false);
  const [thinkingText, setThinkingText] = useState("");

  const [sudoGateOpen, setSudoGateOpen] = useState(false);
  const [sudoGateCommand, setSudoGateCommand] = useState("");
  const [sudoGateReason, setSudoGateReason] = useState("");

  const [accessibilityGranted, setAccessibilityGranted] = useState(true);
  const [windowMode, setWindowMode] = useState<WindowDisplayMode>("panel");

  const scrollRef = useRef<HTMLDivElement>(null);
  const saveChatDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const toStoredMessages = useCallback(
    (msgs: Message[]) =>
      msgs.map((m) => ({
        role: m.role,
        text: m.text,
        commandExecuted: m.commandExecuted,
      })),
    []
  );

  const fromStoredMessages = useCallback(
    (msgs: { role: "user" | "assistant"; text: string; commandExecuted?: string }[]): Message[] =>
      msgs.map((m) => ({
        role: m.role,
        text: m.text,
        commandExecuted: m.commandExecuted,
      })),
    []
  );

  const refreshChatSessionList = useCallback(async () => {
    const store = await getChatStore();
    setChatSessions(store.sessions);
    return store;
  }, []);

  const persistCurrentChat = useCallback(
    async (msgs: Message[]) => {
      if (!activeSessionId) return;
      const summary = await saveChatSession(activeSessionId, toStoredMessages(msgs));
      setChatSessions((prev) => {
        const rest = prev.filter((s) => s.id !== summary.id);
        return [summary, ...rest].sort((a, b) => b.updated_at - a.updated_at);
      });
    },
    [activeSessionId, toStoredMessages]
  );

  const hasUserMessages = messages.some((m) => m.role === "user");
  const showWelcome = !hasUserMessages && !isThinking;

  useEffect(() => {
    scrollRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, isThinking, thinkingText]);

  useEffect(() => {
    if (!prefsLoaded || !activeSessionId || isThinking) return;
    if (saveChatDebounceRef.current) clearTimeout(saveChatDebounceRef.current);
    saveChatDebounceRef.current = setTimeout(() => {
      persistCurrentChat(messages).catch(console.error);
    }, 600);
    return () => {
      if (saveChatDebounceRef.current) clearTimeout(saveChatDebounceRef.current);
    };
  }, [messages, prefsLoaded, activeSessionId, isThinking, persistCurrentChat]);

  const handleNewChat = async () => {
    const session = await createChatSession();
    setActiveSessionId(session.id);
    setMessages(fromStoredMessages(session.messages));
    setInput("");
    await refreshChatSessionList();
    setSettingsOpen(false);
    setKnowledgeOpen(false);
  };

  const handleSelectChat = async (id: string) => {
    if (id === activeSessionId) return;
    await persistCurrentChat(messages);
    await setActiveChatSession(id);
    const session = await loadChatSession(id);
    setActiveSessionId(session.id);
    setMessages(fromStoredMessages(session.messages));
    setSettingsOpen(false);
    setKnowledgeOpen(false);
  };

  const handleDeleteChat = async (id: string) => {
    const wasActive = id === activeSessionId;
    if (wasActive) {
      await persistCurrentChat(messages);
    }
    const next = await deleteChatSession(id);
    await refreshChatSessionList();
    if (wasActive) {
      setActiveSessionId(next.id);
      setMessages(fromStoredMessages(next.messages));
    }
  };

  const refreshLlmAvailability = useCallback(
    async (overrides?: { apiKey?: string; ollamaUrl?: string }) => {
      const savedOllama =
        overrides?.ollamaUrl ?? (await loadCredential("ollama_url"));
      const availability = await resolveLlmAvailability({
        apiKey: overrides?.apiKey ?? apiKey,
        ollamaUrl: savedOllama,
      });
      setLlmAvailability(availability);

      const storedModel = getStoredModel();
      const cloudModel = normalizeCloudModel(storedModel, availability.cloudModels);
      if (cloudModel !== storedModel) {
        setSelectedModel(cloudModel);
        setStoredModel(cloudModel);
      } else {
        setSelectedModel(cloudModel);
      }

      if (apiType === "cloud" && !availability.cloudConfigured && availability.localConfigured) {
        setApiType("local");
        setStoredProvider("local");
      } else if (
        apiType === "local" &&
        !availability.localConfigured &&
        availability.cloudConfigured
      ) {
        setApiType("cloud");
        setStoredProvider("cloud");
        setSelectedModel(cloudModel);
      }

      return availability;
    },
    [apiKey, ollamaUrl, apiType]
  );

  useEffect(() => {
    const init = async () => {
      const provider = getStoredProvider();
      setApiType(provider);
      setSelectedModel(getStoredModel());
      setLocalModel(getStoredLocalModel());

      const env = await getEnvLlmConfig();
      let initApiKey = "";
      if (env.deepseek_api_key) {
        initApiKey = env.deepseek_api_key;
        setApiKey(initApiKey);
        setApiKeySource("env");
        setApiType("cloud");
        setStoredProvider("cloud");
      } else {
        const key = await loadCredential("llm_api_key");
        if (key) {
          initApiKey = key;
          setApiKey(key);
          setApiKeySource("keychain");
        }
      }

      const url = await loadCredential("ollama_url");
      if (url) setOllamaUrl(url);

      const availability = await resolveLlmAvailability({
        apiKey: initApiKey,
        ollamaUrl: url,
      });
      setLlmAvailability(availability);
      const cloudModel = normalizeCloudModel(getStoredModel(), availability.cloudModels);
      setSelectedModel(cloudModel);
      setStoredModel(cloudModel);

      if (provider === "cloud" && !availability.cloudConfigured && availability.localConfigured) {
        setApiType("local");
        setStoredProvider("local");
      } else if (
        provider === "local" &&
        !availability.localConfigured &&
        availability.cloudConfigured
      ) {
        setApiType("cloud");
        setStoredProvider("cloud");
      }

      const configured = await hasProviderConfigured();
      const onboardingDone = getOnboardingComplete();
      if (!configured && !onboardingDone) {
        setShowOnboarding(true);
      }

      const chatStore = await getChatStore();
      setChatSessions(chatStore.sessions);
      if (chatStore.active_id) {
        const session = await loadChatSession(chatStore.active_id);
        setActiveSessionId(session.id);
        setMessages(fromStoredMessages(session.messages));
      }

      setPrefsLoaded(true);
    };
    init();
  }, []);

  useEffect(() => {
    const unlisten = Promise.all([
      listen("menu-open-settings", () => {
        setSettingsOpen(true);
        setKnowledgeOpen(false);
      }),
      listen("menu-open-knowledge", () => {
        setKnowledgeOpen(true);
        setSettingsOpen(false);
      }),
      listen("menu-show-about", () => setAboutOpen(true)),
      listen("menu-new-chat", () => void handleNewChat()),
      listen("menu-cycle-theme", () => cycleTheme()),
    ]);
    return () => {
      unlisten.then((fns) => fns.forEach((fn) => fn()));
    };
  }, []);

  useEffect(() => {
    getWindowMode().then(setWindowMode).catch(console.error);
    let unlisten: (() => void) | undefined;
    subscribeWindowMode(setWindowMode)
      .then((fn) => {
        unlisten = fn;
      })
      .catch(console.error);
    return () => unlisten?.();
  }, []);

  useEffect(() => {
    const scrapeContext = async () => {
      try {
        const windowContext = await invoke<{
          app_name?: string;
          window_title?: string;
        }>("get_active_window");
        if (windowContext) {
          setActiveApp(windowContext.app_name || "Desktop");
          setActiveTitle(windowContext.window_title || "Screen");
        }
      } catch (err) {
        console.error("Window context scraping error:", err);
      }

      try {
        const clip = await invoke<string>("get_clipboard_text");
        if (clip) {
          const trimmed = clip.trim();
          setClipboardSnippet(
            trimmed.length > 30 ? trimmed.substring(0, 30) + "..." : trimmed
          );
        } else {
          setClipboardSnippet("Empty");
        }
      } catch {
        setClipboardSnippet("Unreadable");
      }
    };

    const checkPermissions = async () => {
      try {
        const granted = await invoke<boolean>("check_accessibility_permissions");
        setAccessibilityGranted(granted);
      } catch (err) {
        console.error(err);
      }
    };

    scrapeContext();
    checkPermissions();
    const interval = setInterval(() => {
      scrapeContext();
      checkPermissions();
    }, 2500);
    return () => clearInterval(interval);
  }, []);

  const saveApiKey = async (newKey: string) => {
    setApiKey(newKey);
    setApiKeySource("keychain");
    try {
      await saveCredential("llm_api_key", newKey);
      await refreshLlmAvailability({ apiKey: newKey });
    } catch (err) {
      console.error("Failed to save API key:", err);
    }
  };

  const saveOllamaUrl = async (url: string) => {
    setOllamaUrl(url);
    try {
      await saveCredential("ollama_url", url);
      await refreshLlmAvailability({ ollamaUrl: url });
    } catch (err) {
      console.error("Failed to save Ollama URL:", err);
    }
  };

  const handleOnboardingComplete = (config: {
    provider: ProviderMode;
    apiKey: string;
    ollamaUrl: string;
    cloudModel: string;
    localModel: string;
  }) => {
    setApiType(config.provider);
    setApiKey(config.apiKey);
    setOllamaUrl(config.ollamaUrl);
    setSelectedModel(config.cloudModel);
    setLocalModel(config.localModel);
    setShowOnboarding(false);
    void refreshLlmAvailability({
      apiKey: config.apiKey,
      ollamaUrl: config.ollamaUrl,
    });
  };

  const handleProviderChange = (mode: ProviderMode) => {
    if (mode === "cloud" && !llmAvailability.cloudConfigured) return;
    if (mode === "local" && !llmAvailability.localConfigured) return;
    setApiType(mode);
    setStoredProvider(mode);
    appendUserPreference(`LLM provider set to ${mode}`, "settings").catch(console.error);
  };

  const handleModelChange = (model: string) => {
    if (!model) return;
    if (apiType === "cloud") {
      if (!llmAvailability.cloudModels.some((m) => m.value === model)) return;
      setSelectedModel(model);
      setStoredModel(model);
      appendUserPreference(`Cloud model: ${model}`, "settings").catch(console.error);
    } else {
      setLocalModel(model);
      setStoredLocalModel(model);
      appendUserPreference(`Local Ollama model: ${model}`, "settings").catch(console.error);
    }
  };

  const testElevation = async () => {
    try {
      const res = await invoke<string>("execute_elevated_command", {
        command: "echo 'Success'",
      });
      alert(`Elevation Successful: ${res}`);
    } catch (err) {
      alert(`Elevation Failed: ${err}`);
    }
  };

  const requestPermissions = async () => {
    try {
      await invoke("request_accessibility_permissions");
    } catch (err) {
      console.error(err);
    }
  };

  const resolveSudoGate = useCallback((approved: boolean) => {
    sudoResolveRef.current?.(approved);
    sudoResolveRef.current = null;
    setSudoGateOpen(false);
  }, []);

  const executeCommandSafely = async (command: string): Promise<{
    success: boolean;
    stdout: string;
    stderr: string;
    status_code?: number;
  }> => {
    const safety = await invoke<SafetyCheck>("check_command_safety", { command });

    if (safety.requires_hitl_approval) {
      setSudoGateCommand(command);
      setSudoGateReason(safety.danger_reason || "Safety review requested.");
      setSudoGateOpen(true);

      return new Promise((resolve, reject) => {
        sudoResolveRef.current = async (approved: boolean) => {
          if (!approved) {
            reject(new Error("Command blocked by user in Sudo Gate review."));
            return;
          }
          try {
            if (safety.requires_admin) {
              const out = await invoke<string>("execute_elevated_command", {
                command,
              });
              resolve({ success: true, stdout: out, stderr: "" });
            } else {
              const res = await invoke<{
                success: boolean;
                stdout: string;
                stderr: string;
                status_code?: number;
              }>("execute_shell_command", { command });
              resolve(res);
            }
          } catch (err) {
            reject(err);
          }
        };
      });
    }

    return invoke("execute_shell_command", { command });
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || isThinking) return;

    const userPrompt = input.trim();
    setInput("");
    setMessages((prev) => [...prev, { role: "user", text: userPrompt }]);
    setIsThinking(true);
    setThinkingText(
      apiType === "cloud"
        ? `Calling ${selectedModel}…`
        : `Calling Ollama (${localModel})…`
    );

    try {
      const history = messages
        .filter((m, i) => !(i === 0 && m.role === "assistant"))
        .map((m) => ({
          role: m.role,
          content: m.text,
        }));

      let systemContext = `You are Gnomad ${MUSHROOM}, a helpful desktop assistant by Gnomad Studio.
Active application: ${activeApp}
Active window title: ${activeTitle}
Clipboard preview: ${clipboardSnippet}
Answer the user's question directly and accurately. Use markdown when helpful. Only suggest or run shell commands when the user explicitly asks to automate or execute something on their machine.`;

      try {
        const bundle = await getAgentContextBundle();
        if (bundle.trim()) {
          systemContext += `\n\n---\n\n${trimContextBundle(bundle)}`;
        }
      } catch {
        /* knowledge optional */
      }

      const reply = await chatCompletion({
        provider: apiType,
        model: apiType === "cloud" ? selectedModel : localModel,
        messages: [...history, { role: "user", content: userPrompt }],
        ollamaUrl: ollamaUrl,
        systemContext,
      });

      const nextMessages: Message[] = [
        ...messages,
        { role: "user", text: userPrompt },
        { role: "assistant", text: reply },
      ];
      setMessages(nextMessages);
      await persistCurrentChat(nextMessages);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      const errMessages: Message[] = [
        ...messages,
        { role: "user", text: userPrompt },
        {
          role: "assistant",
          text: `Sorry, I couldn't complete that request: ${msg}`,
        },
      ];
      setMessages(errMessages);
      await persistCurrentChat(errMessages);
    } finally {
      setIsThinking(false);
      setThinkingText("");
    }
  };

  const handleDirectCommand = async () => {
    if (!input.trim()) return;
    const commandToRun = input;
    setInput("");
    setMessages((prev) => [
      ...prev,
      { role: "user", text: `Run CLI: \`${commandToRun}\`` },
    ]);
    setIsThinking(true);
    setThinkingText(`Executing: ${commandToRun}`);

    try {
      const res = await executeCommandSafely(commandToRun);
      setIsThinking(false);
      setThinkingText("");
      setMessages((prev) => [
        ...prev,
        {
          role: "assistant",
          text: res.success
            ? `Success:\n\`\`\`bash\n${res.stdout}\n\`\`\``
            : `Failed:\n\`\`\`bash\n${res.stderr || res.stdout}\n\`\`\``,
          commandExecuted: commandToRun,
          commandResult: {
            success: res.success,
            stdout: res.stdout,
            stderr: res.stderr,
          },
        },
      ]);
    } catch (err: unknown) {
      setIsThinking(false);
      setThinkingText("");
      const msg = err instanceof Error ? err.message : String(err);
      setMessages((prev) => [
        ...prev,
        { role: "assistant", text: `Execution failed: ${msg}` },
      ]);
    }
  };

  const cloudModelOptions: ModelOption[] = llmAvailability.cloudModels;
  const localModelOptions: ModelOption[] = llmAvailability.localModels;
  const headerModelValue =
    apiType === "cloud"
      ? cloudModelOptions.some((m) => m.value === selectedModel)
        ? selectedModel
        : cloudModelOptions[0]?.value ?? ""
      : localModelOptions.some((m) => m.value === localModel)
        ? localModel
        : localModelOptions[0]?.value ?? "";

  const themeIcon =
    themeMode === "light" ? (
      <Sun size={16} />
    ) : themeMode === "dark" ? (
      <Moon size={16} />
    ) : (
      <Monitor size={16} />
    );

  if (!prefsLoaded) {
    return (
      <div className="app-container" data-theme={resolved}>
        <div className="gemini-gradient-bg" />
      </div>
    );
  }

  return (
    <div
      className={`app-container framed mode-${windowMode} ${
        windowMode === "windowed" || windowMode === "fullscreen"
          ? "native-menu-chrome"
          : ""
      }`}
    >
      <div className="gemini-gradient-bg" aria-hidden />

      {showOnboarding && (
        <OnboardingModal onComplete={handleOnboardingComplete} />
      )}

      {aboutOpen && <AboutModal onClose={() => setAboutOpen(false)} />}

      {sudoGateOpen && (
        <div className="modal-overlay">
          <div className="danger-modal">
            <h3 className="danger-title">
              <ShieldAlert size={20} />
              Review command
            </h3>
            <p className="danger-body">
              This command may be risky or need elevated privileges. Approve only if you trust it.
            </p>
            <div className="settings-row">
              <span className="settings-label">Reason</span>
              <div
                style={{
                  color: "var(--color-danger)",
                  fontSize: "0.85rem",
                  fontWeight: 600,
                  display: "flex",
                  alignItems: "center",
                  gap: 6,
                }}
              >
                <AlertCircle size={14} />
                {sudoGateReason}
              </div>
            </div>
            <div className="settings-row">
              <span className="settings-label">Command</span>
              <pre className="cmd-preview">{sudoGateCommand}</pre>
            </div>
            <div className="modal-actions">
              <button
                type="button"
                className="btn secondary"
                onClick={() => resolveSudoGate(false)}
              >
                Deny
              </button>
              <button
                type="button"
                className="btn danger"
                onClick={() => resolveSudoGate(true)}
              >
                Approve
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Windowed: native menu bar (macOS top bar / Linux & Windows in-window menu) */}
      {windowMode !== "windowed" && windowMode !== "fullscreen" && (
        <header className="app-header app-header-gemini">
          <div className="brand-block" title={APP_NAME}>
            <div className="app-logo">
              <GnomadLogo size="sm" />
            </div>
          </div>
          <WindowModeBar mode={windowMode} onModeChange={setWindowMode} compact />
          <div className="header-actions">
            <button
              type="button"
              className="icon-btn"
              onClick={cycleTheme}
              title={`Theme: ${themeMode}`}
              aria-label="Toggle theme"
            >
              {themeIcon}
            </button>
            <button
              type="button"
              className={`icon-btn ${knowledgeOpen ? "active" : ""}`}
              onClick={() => {
                setKnowledgeOpen((o) => !o);
                setSettingsOpen(false);
              }}
              title="Knowledge & skills"
              aria-label="Knowledge and skills"
            >
              <BookOpen size={16} />
            </button>
            <button
              type="button"
              className={`icon-btn ${settingsOpen ? "active" : ""}`}
              onClick={() => {
                setSettingsOpen((o) => !o);
                setKnowledgeOpen(false);
              }}
              title="Settings"
              aria-label="Settings"
            >
              <SettingsIcon size={16} />
            </button>
          </div>
        </header>
      )}

      <div className={`main-layout ${railSide === "right" ? "rail-right" : "rail-left"}`}>
        <ChatSidebar
          sessions={chatSessions}
          activeId={activeSessionId}
          knowledgeOpen={knowledgeOpen}
          onNewChat={() => void handleNewChat()}
          onSelectChat={(id) => void handleSelectChat(id)}
          onDeleteChat={(id) => void handleDeleteChat(id)}
          onOpenKnowledge={() => {
            setKnowledgeOpen(true);
            setSettingsOpen(false);
          }}
          onSideChange={setRailSide}
        />
        <div className="content-pane">
          <div className="context-inline" aria-label="Active context">
            <div className="context-tag" title="Active app">
              <Cpu size={12} />
              <span>{activeApp}</span>
            </div>
            <div className="context-tag" title="Window title">
              <ChevronRight size={12} style={{ color: "var(--color-muted)" }} />
              <span>{activeTitle}</span>
            </div>
            <div className="context-tag" title="Clipboard">
              <Clipboard size={12} />
              <span className="context-tag-mono">{clipboardSnippet}</span>
            </div>
            {!accessibilityGranted && (
              <button
                type="button"
                className="permission-banner"
                onClick={requestPermissions}
              >
                <AlertCircle size={12} />
                <span>Accessibility needed</span>
              </button>
            )}
          </div>

          <div className="chat-scroll">
            {showWelcome && (
              <div className="chat-empty-state">
                <h1 className="welcome-greeting">Hello</h1>
                <p className="welcome-sub">How can I help you today?</p>
              </div>
            )}

            {messages.map((msg, idx) =>
              !showWelcome || msg.role === "user" || idx > 0 ? (
                <div key={`${activeSessionId ?? "chat"}-${idx}`} className={`msg ${msg.role}`}>
                  <span className="msg-header">
                    {msg.role === "user" ? "You" : `${APP_NAME} 🦙`}
                  </span>
                  <span className="msg-text">{msg.text}</span>
                </div>
              ) : null
            )}

            {isThinking && (
              <div className="msg assistant">
                <span
                  className="msg-header"
                  style={{ display: "flex", alignItems: "center", gap: 6 }}
                >
                  <RefreshCw size={12} className="spinning" />
                  Thinking
                </span>
                <div className="thinking-block">{thinkingText}</div>
              </div>
            )}
            <div ref={scrollRef} />
          </div>

          <form className="prompt-footer" onSubmit={handleSubmit}>
            {showWelcome && (
              <div className="suggestion-chips" role="group" aria-label="Suggestions">
                {SUGGESTION_PROMPTS.map((text) => (
                  <button
                    key={text}
                    type="button"
                    className="suggestion-chip"
                    disabled={isThinking}
                    onClick={() => setInput(text)}
                  >
                    {text}
                  </button>
                ))}
              </div>
            )}

            <div className="gemini-composer">
              <input
                className="composer-input"
                value={input}
                onChange={(e) => setInput(e.target.value)}
                placeholder={
                  apiType === "local"
                    ? "Ask Gnomad or type a shell command…"
                    : "Ask Gnomad anything…"
                }
                disabled={isThinking}
              />
              <div className="composer-bar">
                <div className="composer-bar-left">
                  {apiType === "local" && (
                    <button
                      type="button"
                      className="composer-tool-btn"
                      onClick={handleDirectCommand}
                      disabled={isThinking || !input.trim()}
                      title="Run as shell command"
                    >
                      <Terminal size={16} />
                    </button>
                  )}
                </div>
                <div className="composer-bar-right">
                  <select
                    className="composer-select"
                    value={apiType}
                    onChange={(e) => handleProviderChange(e.target.value as ProviderMode)}
                    aria-label="Provider"
                  >
                    <option value="cloud" disabled={!llmAvailability.cloudConfigured}>
                      Cloud
                    </option>
                    <option value="local" disabled={!llmAvailability.localConfigured}>
                      Local
                    </option>
                  </select>
                  <select
                    className="composer-select composer-select-model"
                    value={headerModelValue}
                    onChange={(e) => handleModelChange(e.target.value)}
                    aria-label="Model"
                    disabled={
                      apiType === "cloud"
                        ? cloudModelOptions.length === 0
                        : localModelOptions.length === 0
                    }
                  >
                    {apiType === "cloud" ? (
                      cloudModelOptions.length > 0 ? (
                        cloudModelOptions.map((m) => (
                          <option key={m.value} value={m.value}>
                            {m.label}
                          </option>
                        ))
                      ) : (
                        <option value="">Add API key</option>
                      )
                    ) : localModelOptions.length > 0 ? (
                      localModelOptions.map((m) => (
                        <option key={m.value} value={m.value}>
                          {m.label}
                        </option>
                      ))
                    ) : (
                      <option value="">Set Ollama URL</option>
                    )}
                  </select>
                  <button
                    type="submit"
                    className="composer-send"
                    disabled={isThinking || !input.trim()}
                    aria-label="Send"
                  >
                    <Sparkles size={18} />
                  </button>
                </div>
              </div>
            </div>
          </form>

          {knowledgeOpen && (
            <div className="overlay-pane knowledge-pane">
              <header className="overlay-pane-header">
                <div className="header-left">
                  <BookOpen size={16} style={{ color: "var(--color-accent)" }} />
                  <span className="app-title">Knowledge & skills</span>
                </div>
                <button
                  type="button"
                  className="icon-btn"
                  onClick={() => setKnowledgeOpen(false)}
                  aria-label="Close knowledge"
                >
                  <X size={16} />
                </button>
              </header>
              <div className="overlay-pane-body">
                <KnowledgePanel />
              </div>
            </div>
          )}

          {settingsOpen && (
            <div className="settings-pane">
              <header className="app-header">
                <div className="header-left">
                  <SettingsIcon size={16} style={{ color: "var(--color-accent)" }} />
                  <span className="app-title">Settings</span>
                </div>
                <button
                  type="button"
                  className="icon-btn"
                  onClick={() => setSettingsOpen(false)}
                >
                  <X size={16} />
                </button>
              </header>

              <div className="settings-scroll">
                <div className="settings-section">
                  <h4 className="section-title">
                    <span aria-hidden>🦙</span>
                    Model & API
                  </h4>
                  <div className="settings-row">
                    <span className="settings-label">Provider</span>
                    <select
                      className="settings-select"
                      value={apiType}
                      onChange={(e) =>
                        handleProviderChange(e.target.value as ProviderMode)
                      }
                    >
                      <option value="cloud" disabled={!llmAvailability.cloudConfigured}>
                        Cloud API key
                      </option>
                      <option value="local" disabled={!llmAvailability.localConfigured}>
                        Local Ollama
                      </option>
                    </select>
                  </div>
                  {apiType === "cloud" ? (
                    <>
                      <div className="settings-row">
                        <span className="settings-label">Model</span>
                        <select
                          className="settings-select"
                          value={headerModelValue}
                          onChange={(e) => handleModelChange(e.target.value)}
                          disabled={cloudModelOptions.length === 0}
                        >
                          {cloudModelOptions.length > 0 ? (
                            cloudModelOptions.map((m) => (
                              <option key={m.value} value={m.value}>
                                {m.label}
                              </option>
                            ))
                          ) : (
                            <option value="">Configure DeepSeek API key below</option>
                          )}
                        </select>
                      </div>
                      <div className="settings-row">
                        <span className="settings-label">API key</span>
                        {apiKeySource === "env" && (
                          <p className="knowledge-muted" style={{ marginBottom: 4 }}>
                            🍄 Loaded from project <code>.env</code> (DeepSeek) for testing
                          </p>
                        )}
                        <div className="input-with-icon">
                          <input
                            type={apiKeyVisible ? "text" : "password"}
                            className="settings-input"
                            value={apiKey}
                            onChange={(e) => saveApiKey(e.target.value)}
                            placeholder="sk-... (or use .env DeepSeek_API_KEY)"
                            readOnly={apiKeySource === "env"}
                          />
                          <button
                            type="button"
                            className="icon-btn input-icon-btn"
                            onClick={() => setApiKeyVisible(!apiKeyVisible)}
                          >
                            {apiKeyVisible ? (
                              <EyeOff size={14} />
                            ) : (
                              <Eye size={14} />
                            )}
                          </button>
                        </div>
                      </div>
                      <button
                        type="button"
                        className="btn secondary"
                        onClick={() => setShowOnboarding(true)}
                      >
                        Re-run setup wizard
                      </button>
                    </>
                  ) : (
                    <>
                      <div className="settings-row">
                        <span className="settings-label">Ollama URL</span>
                        <input
                          className="settings-input"
                          value={ollamaUrl}
                          onChange={(e) => saveOllamaUrl(e.target.value)}
                        />
                      </div>
                      <div className="settings-row">
                        <span className="settings-label">Model name</span>
                        <input
                          className="settings-input"
                          value={localModel}
                          onChange={(e) => handleModelChange(e.target.value)}
                        />
                      </div>
                    </>
                  )}
                </div>

                <div className="settings-section">
                  <h4 className="section-title">
                    <span aria-hidden>🍄</span>
                    Knowledge & skills
                  </h4>
                  <p className="knowledge-muted" style={{ marginBottom: 8 }}>
                    Open the <strong>book icon</strong> in the header or use{" "}
                    <strong>Knowledge & skills</strong> in the left sidebar to add
                    files.
                  </p>
                  <button
                    type="button"
                    className="btn secondary"
                    onClick={() => {
                      setSettingsOpen(false);
                      setKnowledgeOpen(true);
                    }}
                  >
                    Open knowledge library
                  </button>
                </div>

                <div className="settings-section">
                  <h4 className="section-title">
                    <ShieldAlert size={16} style={{ color: "var(--color-purple)" }} />
                    Permissions
                  </h4>
                  <div className="settings-row">
                    <span className="settings-label">Accessibility</span>
                    <span
                      style={{
                        fontSize: "0.8rem",
                        fontWeight: 600,
                        color: accessibilityGranted
                          ? "var(--color-success)"
                          : "var(--color-warning)",
                      }}
                    >
                      {accessibilityGranted ? "Granted" : "Required"}
                    </span>
                    {!accessibilityGranted && (
                      <button
                        type="button"
                        className="btn secondary"
                        onClick={requestPermissions}
                      >
                        Open System Settings
                      </button>
                    )}
                  </div>
                  <div className="settings-row">
                    <span className="settings-label">Test elevation</span>
                    <button
                      type="button"
                      className="btn primary"
                      onClick={testElevation}
                    >
                      Trigger admin prompt
                    </button>
                  </div>
                </div>

                <div className="settings-section">
                  <h4 className="section-title">
                    <span aria-hidden>🍄</span>
                    About
                  </h4>
                  <p className="about-credit" style={{ fontSize: "0.85rem" }}>
                    {ABOUT_CREDIT}
                  </p>
                  <p style={{ fontSize: "0.8rem", color: "var(--color-muted)" }}>
                    {APP_NAME} v{VERSION}
                  </p>
                  <button
                    type="button"
                    className="about-link"
                    onClick={() => openUrl(STUDIO_URL)}
                  >
                    {MUSHROOM} gnomadstudio.org
                  </button>
                  <button
                    type="button"
                    className="btn secondary"
                    onClick={() => setAboutOpen(true)}
                  >
                    About Gnomad
                  </button>
                </div>

                <div className="settings-section" style={{ border: "none", background: "transparent" }}>
                  <p style={{ fontSize: "0.75rem", color: "var(--color-muted)", textAlign: "center" }}>
                    {APP_NAME} v{VERSION} · Theme: {resolved}
                  </p>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default App;
