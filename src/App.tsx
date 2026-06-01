import { useState, useEffect, useRef, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { BookOpen, X } from "lucide-react";
import { AboutModal } from "./components/AboutModal";
import { AppToolbar } from "./components/AppToolbar";
import { ChatSidebar } from "./components/ChatSidebar";
import { ChatView } from "./components/ChatView";
import { CursorBloomBackground } from "./components/CursorBloomBackground";
import { KnowledgePanel } from "./components/KnowledgePanel";
import { OnboardingModal } from "./components/OnboardingModal";
import { PathGateModal } from "./components/PathGateModal";
import { SettingsPanel } from "./components/SettingsPanel";
import { SudoGateModal } from "./components/SudoGateModal";
import { useAgentExecution } from "./hooks/useAgentExecution";
import { useChatSessions } from "./hooks/useChatSessions";
import { useChatSubmit } from "./hooks/useChatSubmit";
import { useCursorGlow } from "./hooks/useCursorGlow";
import { useLlmSettings } from "./hooks/useLlmSettings";
import { useOsContext } from "./hooks/useOsContext";
import { useTheme } from "./hooks/useTheme";
import {
  pickChatAttachments,
  removeStagedAttachments,
  type ChatAttachment,
} from "./lib/attachments";
import { getPlatformInfo, type PlatformInfo } from "./lib/platform";
import { getStoredAutoCheckUpdates, getStoredUpdateChannel, setStoredLocalModel, setStoredModel, setStoredProvider, type ProviderMode } from "./lib/preferences";
import { checkForUpdates } from "./lib/updater";
import { pickRandomSuggestions } from "./lib/suggestionPrompts";
import { getStoredSidebarSide, type SidebarSide } from "./lib/sidebarLayout";
import {
  interruptShellSession,
  subscribeShellOutput,
  subscribeShellRunProgress,
} from "./lib/shellSession";
import {
  getWindowMode,
  subscribeWindowMode,
  type WindowDisplayMode,
} from "./lib/windowMode";
import { useAppKeyboard, focusComposerInput } from "./hooks/useAppKeyboard";

function App() {
  const { themeMode, resolved, cycleTheme } = useTheme();
  const appContainerRef = useRef<HTMLDivElement>(null);
  useCursorGlow(appContainerRef);
  const shellOutputBufRef = useRef("");

  const llm = useLlmSettings();
  const [isThinking, setIsThinking] = useState(false);
  const chat = useChatSessions(llm.prefsLoaded, isThinking);

  const [input, setInput] = useState("");
  const [pendingAttachments, setPendingAttachments] = useState<ChatAttachment[]>([]);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [knowledgeOpen, setKnowledgeOpen] = useState(false);
  const [aboutOpen, setAboutOpen] = useState(false);
  const [railSide, setRailSide] = useState<SidebarSide>(getStoredSidebarSide);
  const [thinkingText, setThinkingText] = useState("");
  const [windowMode, setWindowMode] = useState<WindowDisplayMode>("panel");
  const [platformInfo, setPlatformInfo] = useState<PlatformInfo | null>(null);
  const [suggestionChips, setSuggestionChips] = useState(() => pickRandomSuggestions());

  const agent = useAgentExecution({
    ollamaUrl: llm.ollamaUrl,
    apiType: llm.apiType,
    localModel: llm.localModel,
    setThinkingText,
    shellOutputBufRef,
  });

  const os = useOsContext(platformInfo);

  const { handleSubmit, handleDirectCommand } = useChatSubmit({
    messages: chat.messages,
    setMessages: chat.setMessages,
    persistCurrentChat: chat.persistCurrentChat,
    input,
    setInput,
    pendingAttachments,
    setPendingAttachments,
    isThinking,
    setIsThinking,
    setThinkingText,
    apiType: llm.apiType,
    selectedModel: llm.selectedModel,
    localModel: llm.localModel,
    ollamaUrl: llm.ollamaUrl,
    activeApp: os.activeApp,
    activeTitle: os.activeTitle,
    clipboardSnippet: os.clipboardSnippet,
    shellCwd: agent.shellCwd,
    setShellCwd: agent.setShellCwd,
    shellOutputBufRef,
    executeCommandSafely: agent.executeCommandSafely,
    runShellCommandsInChat: agent.runShellCommandsInChat,
    requestHitlApproval: agent.requestHitlApproval,
    requestPathApproval: agent.requestPathApproval,
  });

  const isTrayCompact = windowMode === "panel";
  const showWelcome = chat.hasUserMessages === false && !isThinking;

  useEffect(() => {
    if (showWelcome) setSuggestionChips(pickRandomSuggestions());
  }, [showWelcome, chat.activeSessionId]);

  useEffect(() => {
    const unsubs = Promise.all([
      subscribeShellOutput((chunk) => {
        shellOutputBufRef.current += chunk;
        const lines = shellOutputBufRef.current.split(/\r?\n/);
        const tail = lines.slice(-12).join("\n").trim();
        if (tail) setThinkingText(tail.slice(-500));
      }),
      subscribeShellRunProgress((ev) => {
        if (ev.phase === "running" && ev.command) {
          setThinkingText(`Running: ${ev.command}`);
        } else if (ev.message) {
          setThinkingText(ev.message);
        } else if (ev.phase === "stalled") {
          setThinkingText("Stalled — waiting for terminal output…");
        } else if (ev.phase === "timeout") {
          setThinkingText("Timed out — captured partial output.");
        }
      }),
    ]);
    return () => {
      unsubs.then((fns) => fns.forEach((fn) => fn())).catch(console.error);
    };
  }, []);

  useEffect(() => {
    getPlatformInfo()
      .then((info) => {
        setPlatformInfo(info);
        document.documentElement.dataset.platform = info.os;
      })
      .catch(console.error);
  }, []);

  useEffect(() => {
    const init = async () => {
      await llm.initLlmPrefs();
      await chat.initChatStore();
      if (getStoredAutoCheckUpdates()) {
        checkForUpdates(getStoredUpdateChannel()).catch(() => {
          /* silent startup check */
        });
      }
    };
    void init();
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
      listen("menu-new-chat", () => void onNewChat()),
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

  const onNewChat = async () => {
    if (pendingAttachments.length > 0) {
      void removeStagedAttachments(pendingAttachments.map((a) => a.path));
    }
    setPendingAttachments([]);
    setInput("");
    await chat.handleNewChat(pendingAttachments, () => setPendingAttachments([]));
    setSettingsOpen(false);
    setKnowledgeOpen(false);
  };

  const handleOnboardingComplete = (config: {
    provider: ProviderMode;
    ollamaUrl: string;
    cloudModel: string;
    localModel: string;
  }) => {
    llm.setApiType(config.provider);
    llm.setOllamaUrl(config.ollamaUrl);
    llm.setSelectedModel(config.cloudModel);
    llm.setLocalModel(config.localModel);
    setStoredProvider(config.provider);
    setStoredModel(config.cloudModel);
    setStoredLocalModel(config.localModel);
    llm.setShowOnboarding(false);
    void llm.refreshLlmAvailability({ ollamaUrl: config.ollamaUrl });
  };

  const handleAddAttachments = async () => {
    try {
      const added = await pickChatAttachments();
      if (added.length === 0) return;
      setPendingAttachments((prev) => {
        const seen = new Set(prev.map((a) => a.path));
        return [...prev, ...added.filter((a) => !seen.has(a.path))];
      });
    } catch (err) {
      console.error("Attach files:", err);
    }
  };

  const handleEscape = useCallback(() => {
    if (agent.sudoGateOpen) {
      agent.resolveSudoGate(false);
      return;
    }
    if (agent.pathGateOpen) {
      agent.resolvePathGate(false);
      return;
    }
    if (aboutOpen) {
      setAboutOpen(false);
      return;
    }
    if (llm.showOnboarding) {
      llm.setShowOnboarding(false);
      return;
    }
    if (settingsOpen) {
      setSettingsOpen(false);
      return;
    }
    if (knowledgeOpen) {
      setKnowledgeOpen(false);
    }
  }, [
    agent,
    aboutOpen,
    llm.showOnboarding,
    llm.setShowOnboarding,
    settingsOpen,
    knowledgeOpen,
  ]);

  useAppKeyboard({
    onEscape: handleEscape,
    onFocusComposer: focusComposerInput,
    onOpenSettings: () => {
      setSettingsOpen(true);
      setKnowledgeOpen(false);
    },
    onNewChat: () => void onNewChat(),
    allowShortcuts:
      !agent.sudoGateOpen &&
      !agent.pathGateOpen &&
      !llm.showOnboarding &&
      !aboutOpen,
  });

  const handleRemoveAttachment = (id: string) => {
    setPendingAttachments((prev) => {
      const removed = prev.find((a) => a.id === id);
      if (removed) void removeStagedAttachments([removed.path]);
      return prev.filter((a) => a.id !== id);
    });
  };

  const panelModeTitle =
    platformInfo?.os === "macos" ? "Drop down from menu bar" : "Drop down near system tray";

  const hideIntegratedTitlebar =
    platformInfo?.hideInAppTitlebarWhenWindowed && windowMode === "windowed";

  if (!llm.prefsLoaded) {
    return (
      <div ref={appContainerRef} className="app-container" data-theme={resolved}>
        <CursorBloomBackground />
      </div>
    );
  }

  return (
    <div
      ref={appContainerRef}
      className={`app-container framed mode-${windowMode} titlebar-integrated platform-${
        platformInfo?.os ?? "unknown"
      } ${isTrayCompact ? "view-compact" : "view-expanded"} ${
        windowMode === "windowed" || windowMode === "fullscreen" ? "native-menu-chrome" : ""
      } ${hideIntegratedTitlebar ? "hide-integrated-titlebar" : ""}`}
    >
      <CursorBloomBackground />

      {(windowMode === "windowed" || windowMode === "fullscreen") && (
        <a href="#gnomad-main-content" className="skip-link">
          Skip to main content
        </a>
      )}

      {llm.showOnboarding && <OnboardingModal onComplete={handleOnboardingComplete} />}
      {aboutOpen && <AboutModal onClose={() => setAboutOpen(false)} />}

      <SudoGateModal
        open={agent.sudoGateOpen}
        command={agent.sudoGateCommand}
        reason={agent.sudoGateReason}
        hint={agent.sudoGateHint}
        onDeny={() => agent.resolveSudoGate(false)}
        onApprove={() => agent.resolveSudoGate(true)}
      />
      <PathGateModal
        open={agent.pathGateOpen}
        target={agent.pathGateTarget}
        reason={agent.pathGateReason}
        onDeny={() => agent.resolvePathGate(false)}
        onApprove={() => agent.resolvePathGate(true)}
      />

      <AppToolbar
        platform={platformInfo?.os ?? "macos"}
        windowMode={windowMode}
        panelModeTitle={panelModeTitle}
        onWindowModeChange={setWindowMode}
        themeMode={themeMode}
        onCycleTheme={cycleTheme}
        knowledgeOpen={knowledgeOpen}
        onToggleKnowledge={() => {
          setKnowledgeOpen((o) => !o);
          setSettingsOpen(false);
        }}
        settingsOpen={settingsOpen}
        onToggleSettings={() => {
          setSettingsOpen((o) => !o);
          setKnowledgeOpen(false);
        }}
      />

      <div className={`main-layout ${railSide === "right" ? "rail-right" : "rail-left"}`}>
        <ChatSidebar
          sessions={chat.chatSessions}
          activeId={chat.activeSessionId}
          knowledgeOpen={knowledgeOpen}
          forceCollapsed={isTrayCompact}
          onNewChat={() => void onNewChat()}
          onSelectChat={(id) => {
            void chat.handleSelectChat(id);
            setSettingsOpen(false);
            setKnowledgeOpen(false);
          }}
          onDeleteChat={(id) => void chat.handleDeleteChat(id)}
          onOpenKnowledge={() => {
            setKnowledgeOpen(true);
            setSettingsOpen(false);
          }}
          onSideChange={setRailSide}
        />
        <div id="gnomad-main-content" className="content-pane" tabIndex={-1}>
          <ChatView
            messages={chat.messages}
            activeSessionId={chat.activeSessionId}
            showWelcome={showWelcome}
            isTrayCompact={isTrayCompact}
            isThinking={isThinking}
            thinkingText={thinkingText}
            onStopThinking={() => {
              void interruptShellSession();
              setIsThinking(false);
              setThinkingText("");
            }}
            suggestionChips={suggestionChips}
            onSuggestionClick={setInput}
            input={input}
            onInputChange={setInput}
            onSubmit={handleSubmit}
            onDirectCommand={() => void handleDirectCommand()}
            pendingAttachments={pendingAttachments}
            onAddAttachments={() => void handleAddAttachments()}
            onRemoveAttachment={handleRemoveAttachment}
            apiType={llm.apiType}
            llmAvailability={llm.llmAvailability}
            headerModelValue={llm.headerModelValue}
            onProviderChange={llm.handleProviderChange}
            onModelChange={llm.handleModelChange}
            activeApp={os.activeApp}
            activeTitle={os.activeTitle}
            clipboardSnippet={os.clipboardSnippet}
            accessibilityGranted={os.accessibilityGranted}
            onRequestPermissions={() => void os.requestPermissions()}
          />

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

          <SettingsPanel
            open={settingsOpen}
            onClose={() => setSettingsOpen(false)}
            onOpenAbout={() => setAboutOpen(true)}
            onOpenKnowledge={() => {
              setSettingsOpen(false);
              setKnowledgeOpen(true);
            }}
            onRerunOnboarding={() => llm.setShowOnboarding(true)}
            apiType={llm.apiType}
            headerModelValue={llm.headerModelValue}
            ollamaUrl={llm.ollamaUrl}
            localModel={llm.localModel}
            llmAvailability={llm.llmAvailability}
            shellCwd={agent.shellCwd}
            onShellReset={() => agent.setShellCwd(null)}
            onProviderChange={llm.handleProviderChange}
            onModelChange={llm.handleModelChange}
            onOllamaUrlChange={(url) => void llm.saveOllamaUrl(url)}
            onKeysChanged={() => void llm.refreshLlmAvailability()}
            onAgentSettingsChanged={() => void llm.refreshLlmAvailability()}
            platformInfo={platformInfo}
            accessibilityGranted={os.accessibilityGranted}
            onRequestPermissions={() => void os.requestPermissions()}
            onTestElevation={() => void os.testElevation()}
            resolvedTheme={resolved}
          />
        </div>
      </div>
    </div>
  );
}

export default App;
