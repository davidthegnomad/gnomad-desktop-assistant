import { useEffect, useRef, useState } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { subscribeShellOutput } from "../lib/shellSession";

interface LiveTerminalProps {
  active: boolean;
  /** Subscribe to live PTY chunks (during running commands). */
  stream?: boolean;
  /** Static text for replay (completed commands). */
  initialText?: string;
  className?: string;
}

function summarizeForScreenReader(text: string, maxLen = 120): string {
  const line = text.split(/\r?\n/).filter(Boolean).pop()?.trim() ?? "";
  if (!line) return "";
  return line.length > maxLen ? `${line.slice(0, maxLen - 1)}…` : line;
}

/** xterm.js view for PTY output — live stream or static replay. */
export function LiveTerminal({
  active,
  stream = true,
  initialText,
  className = "",
}: LiveTerminalProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);
  const [liveAnnouncement, setLiveAnnouncement] = useState("");

  useEffect(() => {
    if (!active || !containerRef.current) return;

    const term = new Terminal({
      cursorBlink: stream,
      fontSize: 12,
      fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
      theme: {
        background: "#0d1117",
        foreground: "#e6edf3",
        cursor: "#58a6ff",
      },
      convertEol: true,
      scrollback: 2000,
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    term.open(containerRef.current);
    fit.fit();
    termRef.current = term;

    if (initialText?.trim()) {
      term.write(initialText.replace(/\n/g, "\r\n"));
      if (!initialText.endsWith("\n")) {
        term.write("\r\n");
      }
      const lines = initialText.split(/\r?\n/).filter(Boolean).length;
      setLiveAnnouncement(
        lines > 0
          ? `Command output loaded, ${lines} line${lines === 1 ? "" : "s"}.`
          : "Command output loaded."
      );
    } else if (stream) {
      term.writeln("\x1b[90m— live shell output —\x1b[0m");
      setLiveAnnouncement("Live terminal output started.");
    }

    let resizeRaf = 0;
    const onResize = () => {
      if (resizeRaf) cancelAnimationFrame(resizeRaf);
      resizeRaf = requestAnimationFrame(() => {
        resizeRaf = 0;
        try {
          if (containerRef.current?.offsetWidth && containerRef.current?.offsetHeight) {
            fit.fit();
          }
        } catch {
          /* hidden */
        }
      });
    };
    window.addEventListener("resize", onResize);
    const ro = new ResizeObserver(onResize);
    ro.observe(containerRef.current);

    let unlisten: (() => void) | undefined;
    let announceTimer: ReturnType<typeof setTimeout> | undefined;
    let pendingChunk = "";

    if (stream) {
      void subscribeShellOutput((chunk) => {
        term.write(chunk);
        pendingChunk += chunk;
        if (announceTimer) return;
        announceTimer = setTimeout(() => {
          announceTimer = undefined;
          const summary = summarizeForScreenReader(pendingChunk);
          pendingChunk = "";
          if (summary) {
            setLiveAnnouncement(`Terminal: ${summary}`);
          }
        }, 2000);
      }).then((fn) => {
        unlisten = fn;
      });
    }

    return () => {
      if (resizeRaf) cancelAnimationFrame(resizeRaf);
      window.removeEventListener("resize", onResize);
      ro.disconnect();
      if (announceTimer) clearTimeout(announceTimer);
      unlisten?.();
      term.dispose();
      termRef.current = null;
    };
  }, [active, stream, initialText]);

  if (!active) return null;

  return (
    <div
      className={`live-terminal-wrap ${className}`.trim()}
      role="region"
      aria-label="Terminal output"
    >
      <div
        className="sr-only"
        aria-live="polite"
        aria-atomic="true"
      >
        {liveAnnouncement}
      </div>
      <div ref={containerRef} className="live-terminal-host" aria-hidden="true" />
    </div>
  );
}
