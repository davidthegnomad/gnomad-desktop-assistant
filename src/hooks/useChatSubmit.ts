import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AGENT_SYSTEM_TOOLS, runAgentLoop } from "../lib/agentLoop";
import { AGENT_SHELL_SYSTEM_APPEND } from "../lib/agentShell";
import { getAgentSettings } from "../lib/agentSettings";
import {
  collectInvalidGnomadRunTags,
  extractCommandsFromAssistantReply,
  extractUserShellCommand,
  stripExecutableBlocksFromReply,
  userWantsShellExecution,
} from "../lib/agentShell";
import {
  formatAttachmentsForPrompt,
  type ChatAttachment,
} from "../lib/attachments";
import { executionFailedLabel, parseInvokeError } from "../lib/errors";
import { getAgentContextBundle, trimContextBundle } from "../lib/knowledge";
import { chatCompletion } from "../lib/llm";
import { MUSHROOM } from "../lib/brand";
import type { ProviderMode } from "../lib/preferences";
import type { Message } from "../types/chat";

export function useChatSubmit(options: {
  messages: Message[];
  setMessages: React.Dispatch<React.SetStateAction<Message[]>>;
  persistCurrentChat: (msgs: Message[]) => Promise<void>;
  input: string;
  setInput: (v: string) => void;
  pendingAttachments: ChatAttachment[];
  setPendingAttachments: React.Dispatch<React.SetStateAction<ChatAttachment[]>>;
  isThinking: boolean;
  setIsThinking: (v: boolean) => void;
  setThinkingText: (t: string) => void;
  apiType: ProviderMode;
  selectedModel: string;
  localModel: string;
  ollamaUrl: string;
  activeApp: string;
  activeTitle: string;
  clipboardSnippet: string;
  shellCwd: string | null;
  setShellCwd: (cwd: string | null) => void;
  shellOutputBufRef: React.MutableRefObject<string>;
  executeCommandSafely: (command: string) => Promise<NonNullable<Message["commandResult"]> & { cwd?: string }>;
  runShellCommandsInChat: (
    commands: string[],
    base: Message[],
    invalid?: string[],
    agentSettings?: Awaited<ReturnType<typeof getAgentSettings>>
  ) => Promise<Message[]>;
  requestHitlApproval: (command: string, reason: string) => Promise<string | null>;
  requestPathApproval: (path: string, reason: string) => Promise<boolean>;
}) {
  const handleSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      const userPrompt = options.input.trim();
      const attachSnapshot = [...options.pendingAttachments];
      if ((!userPrompt && attachSnapshot.length === 0) || options.isThinking) return;

      const runIntent = userWantsShellExecution(userPrompt);
      const directCmd = extractUserShellCommand(userPrompt);

      options.setInput("");
      options.setPendingAttachments([]);

      let attachmentBlock = "";
      try {
        attachmentBlock = await formatAttachmentsForPrompt(
          attachSnapshot.map((a) => a.path)
        );
      } catch (err) {
        console.error("Format attachments:", err);
      }

      const userContent = attachmentBlock
        ? userPrompt
          ? `${userPrompt}\n\n${attachmentBlock}`
          : attachmentBlock
        : userPrompt;

      const displayText = userPrompt
        ? attachSnapshot.length > 0
          ? `${userPrompt}\n\n📎 ${attachSnapshot.map((a) => a.name).join(", ")}`
          : userPrompt
        : `📎 ${attachSnapshot.map((a) => a.name).join(", ")}`;

      const userMessage: Message = {
        role: "user",
        text: displayText,
        attachments: attachSnapshot.length > 0 ? attachSnapshot : undefined,
      };

      options.setMessages((prev) => [...prev, userMessage]);
      options.setIsThinking(true);

      if (directCmd) {
        options.shellOutputBufRef.current = "";
        options.setThinkingText(`Running: ${directCmd}`);
        try {
          const res = await options.executeCommandSafely(directCmd);
          if (res.cwd) options.setShellCwd(res.cwd);
          const nextMessages: Message[] = [
            ...options.messages,
            userMessage,
            {
              role: "assistant",
              text: res.message ?? (res.success ? "Command finished." : "Command failed."),
              commandExecuted: directCmd,
              commandResult: res,
            },
          ];
          options.setMessages(nextMessages);
          await options.persistCurrentChat(nextMessages);
        } catch (err: unknown) {
          const nextMessages: Message[] = [
            ...options.messages,
            userMessage,
            {
              role: "assistant",
              text: executionFailedLabel(err),
              commandExecuted: directCmd,
              errorPayload: parseInvokeError(err) ?? undefined,
            },
          ];
          options.setMessages(nextMessages);
          await options.persistCurrentChat(nextMessages);
        } finally {
          options.setIsThinking(false);
          options.setThinkingText("");
        }
        return;
      }

      options.setThinkingText(
        options.apiType === "cloud"
          ? `Calling ${options.selectedModel}…`
          : `Calling Ollama (${options.localModel})…`
      );

      try {
        const history = options.messages
          .filter((m, i) => !(i === 0 && m.role === "assistant"))
          .map((m) => ({ role: m.role, content: m.text }));

        let systemContext = `You are Gnomad ${MUSHROOM}, a helpful desktop assistant by Gnomad Studio.
Active application: ${options.activeApp}
Active window title: ${options.activeTitle}
Clipboard preview: ${options.clipboardSnippet}
Shell working directory: ${options.shellCwd ?? "(session default)"}
Answer the user's question directly and accurately. Use markdown when helpful.

${AGENT_SYSTEM_TOOLS}`;

        try {
          const bundle = await getAgentContextBundle();
          if (bundle.trim()) {
            systemContext += `\n\n---\n\n${trimContextBundle(bundle)}`;
          }
        } catch {
          /* optional */
        }

        let nextMessages: Message[] = [...options.messages, userMessage];
        const agentSettings = await getAgentSettings();

        if (options.apiType === "cloud") {
          options.setMessages(nextMessages);
          const loopResult = await runAgentLoop({
            provider: options.apiType,
            model: options.selectedModel,
            ollamaUrl: options.ollamaUrl,
            systemContext,
            messages: [...history, { role: "user", content: userContent }],
            shellCwd: options.shellCwd ?? undefined,
            enableTools: true,
            agentSettings,
            chatLocalModel: options.localModel,
            callbacks: {
              onStep: (_step, label) => options.setThinkingText(label),
              requestHitlApproval: options.requestHitlApproval,
              requestPathApproval: options.requestPathApproval,
              executeElevated: (command, approvalToken) =>
                invoke<string>("execute_elevated_command", { command, approvalToken }),
            },
          });

          for (const action of loopResult.actions) {
            if (action.commandResult?.cwd) {
              options.setShellCwd(action.commandResult.cwd);
            }
            nextMessages = [
              ...nextMessages,
              {
                role: "assistant",
                text: action.label,
                commandExecuted: action.commandExecuted,
                commandResult: action.commandResult,
                errorPayload: action.errorPayload,
              },
            ];
          }
          nextMessages = [
            ...nextMessages,
            { role: "assistant", text: loopResult.finalText },
          ];
        } else {
          systemContext += `\n\n${AGENT_SHELL_SYSTEM_APPEND}`;
          const reply = await chatCompletion({
            provider: options.apiType,
            model: options.localModel,
            messages: [...history, { role: "user", content: userContent }],
            ollamaUrl: options.ollamaUrl,
            systemContext,
          });

          const commands = extractCommandsFromAssistantReply(reply, runIntent);
          const invalidFromModel = collectInvalidGnomadRunTags(reply);
          const intro = stripExecutableBlocksFromReply(reply);

          nextMessages = [
            ...nextMessages,
            ...(intro
              ? [{ role: "assistant" as const, text: intro }]
              : commands.length > 0 || invalidFromModel.length > 0
                ? [{ role: "assistant" as const, text: "Running command(s) in your shell session…" }]
                : [{ role: "assistant" as const, text: reply }]),
          ];

          if (commands.length > 0 || invalidFromModel.length > 0) {
            options.setMessages(nextMessages);
            nextMessages = await options.runShellCommandsInChat(
              commands,
              nextMessages,
              invalidFromModel,
              agentSettings
            );
          }
        }

        options.setMessages(nextMessages);
        await options.persistCurrentChat(nextMessages);
      } catch (err: unknown) {
        const errMessages: Message[] = [
          ...options.messages,
          {
            role: "user",
            text: displayText,
            attachments: attachSnapshot.length > 0 ? attachSnapshot : undefined,
          },
          {
            role: "assistant",
            text: `Sorry, I couldn't complete that request: ${executionFailedLabel(err).replace(/^Execution failed: /, "")}`,
            errorPayload: parseInvokeError(err) ?? undefined,
          },
        ];
        options.setMessages(errMessages);
        await options.persistCurrentChat(errMessages);
      } finally {
        options.setIsThinking(false);
        options.setThinkingText("");
      }
    },
    [options]
  );

  const handleDirectCommand = useCallback(async () => {
    if (!options.input.trim()) return;
    const commandToRun = options.input;
    options.setInput("");
    options.setMessages((prev) => [
      ...prev,
      { role: "user", text: `Run CLI: \`${commandToRun}\`` },
    ]);
    options.setIsThinking(true);
    options.shellOutputBufRef.current = "";
    options.setThinkingText(`Executing: ${commandToRun}`);

    try {
      const res = await options.executeCommandSafely(commandToRun);
      if (res.cwd) options.setShellCwd(res.cwd);
      options.setIsThinking(false);
      options.setThinkingText("");
      options.setMessages((prev) => [
        ...prev,
        {
          role: "assistant",
          text: res.message ?? (res.success ? "Command finished." : "Command failed."),
          commandExecuted: commandToRun,
          commandResult: res,
        },
      ]);
    } catch (err: unknown) {
      options.setIsThinking(false);
      options.setThinkingText("");
      options.setMessages((prev) => [
        ...prev,
        {
          role: "assistant",
          text: executionFailedLabel(err),
          commandExecuted: commandToRun,
          errorPayload: parseInvokeError(err) ?? undefined,
        },
      ]);
    }
  }, [options]);

  return { handleSubmit, handleDirectCommand };
}
