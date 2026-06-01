import { useCallback, useEffect, useRef, useState } from "react";
import {
  BookOpen,
  GripVertical,
  MessageSquarePlus,
  PanelLeftClose,
  PanelLeftOpen,
  PanelRightClose,
} from "lucide-react";
import { ChatHistoryPanel } from "./ChatHistoryPanel";
import { GnomadLogo } from "./GnomadLogo";
import { StudioLink } from "./StudioLink";
import type { ChatSessionSummary } from "../lib/chatHistory";
import {
  SIDEBAR_COLLAPSED_WIDTH,
  SIDEBAR_SWAP_DRAG_PX,
  SIDEBAR_WIDTH_MAX,
  SIDEBAR_WIDTH_MIN,
  getStoredSidebarCollapsed,
  getStoredSidebarSide,
  getStoredSidebarWidth,
  setStoredSidebarCollapsed,
  setStoredSidebarSide,
  setStoredSidebarWidth,
  type SidebarSide,
} from "../lib/sidebarLayout";

interface ChatSidebarProps {
  sessions: ChatSessionSummary[];
  activeId: string | null;
  knowledgeOpen: boolean;
  /** Tray panel: keep icon rail only. */
  forceCollapsed?: boolean;
  onNewChat: () => void;
  onSelectChat: (id: string) => void;
  onDeleteChat: (id: string) => void;
  onOpenKnowledge: () => void;
  onSideChange?: (side: SidebarSide) => void;
}

export function ChatSidebar({
  sessions,
  activeId,
  knowledgeOpen,
  forceCollapsed = false,
  onNewChat,
  onSelectChat,
  onDeleteChat,
  onOpenKnowledge,
  onSideChange,
}: ChatSidebarProps) {
  const [collapsed, setCollapsed] = useState(getStoredSidebarCollapsed);
  const [side, setSide] = useState<SidebarSide>(getStoredSidebarSide);
  const [width, setWidth] = useState(getStoredSidebarWidth);

  const resizingRef = useRef(false);
  const swapDragRef = useRef<{ startX: number } | null>(null);

  useEffect(() => {
    onSideChange?.(side);
  }, [side, onSideChange]);

  const effectiveCollapsed = collapsed || forceCollapsed;
  const railWidth = effectiveCollapsed ? SIDEBAR_COLLAPSED_WIDTH : width;

  const toggleCollapsed = useCallback(() => {
    setCollapsed((c) => {
      const next = !c;
      setStoredSidebarCollapsed(next);
      return next;
    });
  }, []);

  const flipSide = useCallback(() => {
    setSide((s) => {
      const next: SidebarSide = s === "left" ? "right" : "left";
      setStoredSidebarSide(next);
      return next;
    });
  }, []);

  useEffect(() => {
    const onMove = (e: PointerEvent) => {
      if (resizingRef.current && !collapsed) {
        const delta = side === "left" ? e.clientX : window.innerWidth - e.clientX;
        const next = Math.min(
          SIDEBAR_WIDTH_MAX,
          Math.max(SIDEBAR_WIDTH_MIN, Math.round(delta))
        );
        setWidth(next);
        setStoredSidebarWidth(next);
        return;
      }

    };

    const onUp = (e: PointerEvent) => {
      const drag = swapDragRef.current;
      if (drag) {
        const dx = e.clientX - drag.startX;
        if (side === "left" && dx >= SIDEBAR_SWAP_DRAG_PX) flipSide();
        if (side === "right" && dx <= -SIDEBAR_SWAP_DRAG_PX) flipSide();
      }
      resizingRef.current = false;
      swapDragRef.current = null;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };

    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    window.addEventListener("pointercancel", onUp);
    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      window.removeEventListener("pointercancel", onUp);
    };
  }, [collapsed, side, flipSide]);

  const startResize = (e: React.PointerEvent) => {
    if (collapsed) return;
    e.preventDefault();
    resizingRef.current = true;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
  };

  const startSwapDrag = (e: React.PointerEvent) => {
    if (collapsed) return;
    e.preventDefault();
    swapDragRef.current = { startX: e.clientX };
    document.body.style.userSelect = "none";
  };

  const CollapseIcon = side === "left" ? PanelLeftClose : PanelRightClose;
  const ExpandIcon = PanelLeftOpen;

  const brandHeader = (
    <div className="side-rail-brand" title="Gnomad Studio">
      <GnomadLogo size="sm" />
      <StudioLink />
    </div>
  );

  return (
    <aside
      className={`side-rail rail-${side} ${effectiveCollapsed ? "collapsed" : ""}`}
      style={{ width: railWidth, minWidth: railWidth, maxWidth: railWidth }}
      data-side={side}
    >
      {effectiveCollapsed ? (
        <div className="side-rail-compact">
          {brandHeader}
          <button
            type="button"
            className="side-rail-icon-btn"
            title="New chat"
            aria-label="New chat"
            onClick={onNewChat}
          >
            <MessageSquarePlus size={18} />
          </button>
          <button
            type="button"
            className={`side-rail-icon-btn ${knowledgeOpen ? "active" : ""}`}
            title="Knowledge & skills"
            aria-label="Knowledge and skills"
            onClick={onOpenKnowledge}
          >
            <BookOpen size={18} />
          </button>
          <button
            type="button"
            className="side-rail-icon-btn side-rail-expand"
            title="Expand chats pane"
            aria-label="Expand chats pane"
            onClick={toggleCollapsed}
          >
            <ExpandIcon size={18} />
          </button>
        </div>
      ) : (
        <>
          {brandHeader}
          <div
            className="side-rail-chrome"
            onPointerDown={startSwapDrag}
            title="Drag toward the other side to move this pane"
          >
            <GripVertical size={14} className="side-rail-grip" aria-hidden />
            <span className="side-rail-title">Chats</span>
            <div className="side-rail-chrome-actions">
              <button
                type="button"
                className="icon-btn"
                title="New chat"
                aria-label="New chat"
                onPointerDown={(e) => e.stopPropagation()}
                onClick={onNewChat}
              >
                <MessageSquarePlus size={16} />
              </button>
              <button
                type="button"
                className="icon-btn"
                title="Collapse pane"
                aria-label="Collapse chats pane"
                onPointerDown={(e) => e.stopPropagation()}
                onClick={toggleCollapsed}
              >
                <CollapseIcon size={16} />
              </button>
            </div>
          </div>

          <ChatHistoryPanel
            sessions={sessions}
            activeId={activeId}
            onSelect={onSelectChat}
            onDelete={onDeleteChat}
            hideHeader
          />

          <button
            type="button"
            className={`side-rail-knowledge-btn ${knowledgeOpen ? "active" : ""}`}
            onClick={onOpenKnowledge}
          >
            <BookOpen size={15} />
            <span>Knowledge & skills</span>
          </button>

          <div
            className="side-rail-resizer"
            onPointerDown={startResize}
            role="separator"
            aria-orientation="vertical"
            aria-label="Resize chats pane"
            title="Drag to resize"
          />
        </>
      )}
    </aside>
  );
}
