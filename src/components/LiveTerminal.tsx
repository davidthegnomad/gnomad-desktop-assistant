import { useEffect, useRef } from "react";
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

/** xterm.js view for PTY output — live stream or static replay. */
export function LiveTerminal({
  active,
  stream = true,
  initialText,
  className = "",
}: LiveTerminalProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);

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
    } else if (stream) {
      term.writeln("\x1b[90m— live shell output —\x1b[0m");
    }

    const onResize = () => {
      try {
        fit.fit();
      } catch {
        /* hidden */
      }
    };
    window.addEventListener("resize", onResize);
    const ro = new ResizeObserver(onResize);
    ro.observe(containerRef.current);

    let unlisten: (() => void) | undefined;
    if (stream) {
      void subscribeShellOutput((chunk) => {
        term.write(chunk);
      }).then((fn) => {
        unlisten = fn;
      });
    }

    return () => {
      window.removeEventListener("resize", onResize);
      ro.disconnect();
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
      <div ref={containerRef} className="live-terminal-host" />
    </div>
  );
}
