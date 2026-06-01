import { useCallback, useEffect, useState } from "react";
import { Brain, FolderOpen, Shield, Zap } from "lucide-react";
import {
  getAgentSettings,
  pickWorkspaceFolder,
  setCommandPlanner,
  setTrustMode,
  type AgentSettings,
  type TrustMode,
} from "../lib/agentSettings";
import { pickGgufFile } from "../lib/commandPlanner";

export function AgentAccessSettings() {
  const [settings, setSettings] = useState<AgentSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [plannerModel, setPlannerModel] = useState("llama3.2:1b");
  const [plannerGguf, setPlannerGguf] = useState("");

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const s = await getAgentSettings();
      setSettings(s);
      setPlannerModel(s.commandPlannerModel || "llama3.2:1b");
      setPlannerGguf(s.commandPlannerGgufPath || "");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const onTrustChange = async (mode: TrustMode) => {
    setSettings(await setTrustMode(mode));
  };

  const savePlanner = async (patch: Partial<{
    enabled: boolean;
    model: string;
    useChatLocalModel: boolean;
    ggufPath: string;
  }>) => {
    if (!settings) return;
    const next = await setCommandPlanner({
      enabled: patch.enabled ?? settings.commandPlannerEnabled,
      model: patch.model ?? plannerModel,
      useChatLocalModel:
        patch.useChatLocalModel ?? settings.commandPlannerUseChatLocalModel,
      ggufPath: patch.ggufPath ?? plannerGguf,
    });
    setSettings(next);
    setPlannerModel(next.commandPlannerModel);
    setPlannerGguf(next.commandPlannerGgufPath);
  };

  return (
    <div className="settings-section agent-access-section">
      <h4 className="section-title">
        <Shield size={16} />
        Agent access
      </h4>
      <p className="knowledge-muted">
        Workspace defaults to your home folder. The agent can read, write, and run
        commands inside it. Risky shell commands still use Sudo Gate.
      </p>

      {loading ? (
        <p className="knowledge-muted">Loading…</p>
      ) : settings ? (
        <>
          <div className="agent-workspace-row">
            <span className="agent-workspace-label">Workspace</span>
            <code className="agent-workspace-path" title={settings.workspaceRoot}>
              {settings.workspaceRoot}
            </code>
            <button
              type="button"
              className="btn-secondary btn-sm"
              onClick={() => void pickWorkspaceFolder().then((s) => s && setSettings(s))}
            >
              <FolderOpen size={14} />
              Change…
            </button>
          </div>

          <div className="agent-trust-options">
            <label
              className="agent-trust-option"
              title="Read, write, and run commands in your workspace. Paths outside it require approval. Destructive shell commands use Sudo Gate."
            >
              <input
                type="radio"
                name="trustMode"
                checked={settings.trustMode === "standard"}
                onChange={() => void onTrustChange("standard")}
              />
              <span>
                <strong>Standard</strong> — workspace read/write + gated shell
              </span>
            </label>
            <label
              className="agent-trust-option agent-trust-yolo"
              title="Fewer path prompts: the agent can read or write anywhere your user account can. Sudo Gate may still appear for dangerous shell patterns. Only enable if you trust the model and keep backups."
            >
              <input
                type="radio"
                name="trustMode"
                checked={settings.trustMode === "yolo"}
                onChange={() => void onTrustChange("yolo")}
              />
              <span>
                <Zap size={14} />
                <strong>YOLO!</strong> — full machine file access
              </span>
            </label>
          </div>

          <div className="agent-planner-block">
            <h4 className="section-title subsection-title">
              <Brain size={16} />
              Local command planner
            </h4>
            <p className="knowledge-muted">
              When the main model outputs prose instead of a real command, a small
              local model can rewrite it into CLI (via Ollama). Works with cloud
              chat too — planner always runs locally.
            </p>
            <label
              className="agent-trust-option"
              title="Use a fast local model to convert intents like “check if brew is installed” into command -v brew before execution."
            >
              <input
                type="checkbox"
                checked={settings.commandPlannerEnabled}
                onChange={(e) => void savePlanner({ enabled: e.target.checked })}
              />
              <span>
                <strong>Enable command planner</strong>
              </span>
            </label>

            {settings.commandPlannerEnabled && (
              <div className="agent-planner-fields">
                <label
                  className="agent-trust-option"
                  title="When you use Local (Ollama) chat, reuse the same model name as chat instead of the planner model below. Not recommended for large chat models — use a small dedicated planner model instead."
                >
                  <input
                    type="checkbox"
                    checked={settings.commandPlannerUseChatLocalModel}
                    onChange={(e) =>
                      void savePlanner({ useChatLocalModel: e.target.checked })
                    }
                  />
                  <span>Use chat local model as planner (local mode only)</span>
                </label>

                <div className="settings-row">
                  <span
                    className="settings-label"
                    title="Ollama model tag for the planner, e.g. llama3.2:1b or phi3:mini. Should be small and fast — separate from your main DeepSeek or large Ollama chat model."
                  >
                    Planner model (Ollama)
                  </span>
                  <input
                    className="settings-input"
                    value={plannerModel}
                    onChange={(e) => setPlannerModel(e.target.value)}
                    onBlur={() => void savePlanner({ model: plannerModel })}
                    placeholder="llama3.2:1b"
                    disabled={settings.commandPlannerUseChatLocalModel}
                  />
                </div>

                <div className="settings-row agent-gguf-row">
                  <span
                    className="settings-label"
                    title="Optional path to a .gguf weights file for future direct inference. Today the planner uses Ollama only — import GGUF with `ollama create` or use the model name field above."
                  >
                    GGUF path (optional)
                  </span>
                  <input
                    className="settings-input"
                    value={plannerGguf}
                    onChange={(e) => setPlannerGguf(e.target.value)}
                    onBlur={() => void savePlanner({ ggufPath: plannerGguf })}
                    placeholder="/path/to/model.gguf"
                  />
                  <button
                    type="button"
                    className="btn-secondary btn-sm"
                    onClick={() =>
                      void pickGgufFile().then((p) => {
                        if (p) {
                          setPlannerGguf(p);
                          void savePlanner({ ggufPath: p });
                        }
                      })
                    }
                  >
                    Browse…
                  </button>
                </div>
              </div>
            )}
          </div>
        </>
      ) : null}
    </div>
  );
}
