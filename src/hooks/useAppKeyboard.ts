import { useEffect, useRef } from "react";

export interface AppKeyboardHandlers {
  onEscape?: () => void;
  onFocusComposer?: () => void;
  onOpenSettings?: () => void;
  onNewChat?: () => void;
  /** When false, only Escape is handled (e.g. during gate modals). */
  allowShortcuts?: boolean;
}

function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName;
  return (
    tag === "INPUT" ||
    tag === "TEXTAREA" ||
    tag === "SELECT" ||
    target.isContentEditable
  );
}

export function useAppKeyboard(handlers: AppKeyboardHandlers) {
  const handlersRef = useRef(handlers);
  handlersRef.current = handlers;

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      const h = handlersRef.current;
      if (e.key === "Escape") {
        h.onEscape?.();
        return;
      }

      if (!h.allowShortcuts) return;
      if (isTypingTarget(e.target) && !(e.metaKey || e.ctrlKey)) return;

      const mod = e.metaKey || e.ctrlKey;
      if (!mod) return;

      const key = e.key.toLowerCase();
      if (key === "k" || key === "l") {
        e.preventDefault();
        h.onFocusComposer?.();
      } else if (key === ",") {
        e.preventDefault();
        h.onOpenSettings?.();
      } else if (key === "n") {
        e.preventDefault();
        h.onNewChat?.();
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);
}

export function focusComposerInput() {
  const el = document.getElementById("gnomad-composer-input");
  if (el instanceof HTMLElement) {
    el.focus();
  }
}
