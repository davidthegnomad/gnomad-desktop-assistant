export type SidebarSide = "left" | "right";

const LS_COLLAPSED = "gnomad_sidebar_collapsed";
const LS_SIDE = "gnomad_sidebar_side";
const LS_WIDTH = "gnomad_sidebar_width";

export const SIDEBAR_WIDTH_MIN = 160;
export const SIDEBAR_WIDTH_MAX = 420;
export const SIDEBAR_WIDTH_DEFAULT = 220;
export const SIDEBAR_COLLAPSED_WIDTH = 52;
export const SIDEBAR_SWAP_DRAG_PX = 100;

export function getStoredSidebarCollapsed(): boolean {
  return localStorage.getItem(LS_COLLAPSED) === "true";
}

export function setStoredSidebarCollapsed(collapsed: boolean) {
  localStorage.setItem(LS_COLLAPSED, collapsed ? "true" : "false");
}

export function getStoredSidebarSide(): SidebarSide {
  return localStorage.getItem(LS_SIDE) === "right" ? "right" : "left";
}

export function setStoredSidebarSide(side: SidebarSide) {
  localStorage.setItem(LS_SIDE, side);
}

export function getStoredSidebarWidth(): number {
  const n = Number(localStorage.getItem(LS_WIDTH));
  if (!Number.isFinite(n)) return SIDEBAR_WIDTH_DEFAULT;
  return Math.min(SIDEBAR_WIDTH_MAX, Math.max(SIDEBAR_WIDTH_MIN, Math.round(n)));
}

export function setStoredSidebarWidth(width: number) {
  localStorage.setItem(LS_WIDTH, String(width));
}
