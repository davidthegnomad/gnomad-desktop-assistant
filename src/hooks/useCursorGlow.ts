import { useEffect, useRef, type RefObject } from "react";

const LERP = 0.12;
const IDLE_CENTER = { x: 50, y: 50 };

/**
 * Subtle cursor bloom: soft center only, gentle breathe + follow.
 */
export function useCursorGlow(
  containerRef: RefObject<HTMLElement | null>,
  enabled = true
) {
  const target = useRef(IDLE_CENTER);
  const current = useRef(IDLE_CENTER);
  const prev = useRef(IDLE_CENTER);
  const rafId = useRef<number>(0);
  const startTime = useRef(performance.now());

  useEffect(() => {
    if (!enabled) return;
    const el = containerRef.current;
    if (!el) return;

    const onPointerMove = (e: PointerEvent) => {
      const rect = el.getBoundingClientRect();
      if (rect.width <= 0 || rect.height <= 0) return;
      target.current = {
        x: ((e.clientX - rect.left) / rect.width) * 100,
        y: ((e.clientY - rect.top) / rect.height) * 100,
      };
    };

    const onPointerLeave = () => {
      target.current = IDLE_CENTER;
    };

    const tick = () => {
      prev.current = { ...current.current };
      current.current = {
        x: current.current.x + (target.current.x - current.current.x) * LERP,
        y: current.current.y + (target.current.y - current.current.y) * LERP,
      };

      const t = (performance.now() - startTime.current) / 1000;
      const breathe = Math.sin(t * 1.8) * 0.05 + Math.sin(t * 2.9) * 0.03;
      const frameVel = Math.hypot(
        current.current.x - prev.current.x,
        current.current.y - prev.current.y
      );
      const chase = Math.hypot(
        target.current.x - current.current.x,
        target.current.y - current.current.y
      );
      const moveBoost = Math.min(frameVel * 0.15 + chase * 0.025, 0.12);
      const scale = 1 + breathe + moveBoost;
      const strength = 0.38 + Math.sin(t * 1.4) * 0.08 + moveBoost * 0.25;
      const hue = (t * 35) % 360;

      el.style.setProperty("--cursor-glow-x", `${current.current.x}%`);
      el.style.setProperty("--cursor-glow-y", `${current.current.y}%`);
      el.style.setProperty("--cursor-glow-scale", scale.toFixed(3));
      el.style.setProperty("--cursor-glow-strength", strength.toFixed(3));
      el.style.setProperty("--cursor-glow-hue", `${hue}deg`);
      el.style.setProperty(
        "--cursor-glow-x2",
        `${current.current.x + Math.sin(t * 1.2) * 2}%`
      );
      el.style.setProperty(
        "--cursor-glow-y2",
        `${current.current.y + Math.cos(t * 1.5) * 2}%`
      );

      rafId.current = requestAnimationFrame(tick);
    };

    el.addEventListener("pointermove", onPointerMove);
    el.addEventListener("pointerleave", onPointerLeave);
    rafId.current = requestAnimationFrame(tick);

    return () => {
      el.removeEventListener("pointermove", onPointerMove);
      el.removeEventListener("pointerleave", onPointerLeave);
      cancelAnimationFrame(rafId.current);
    };
  }, [containerRef, enabled]);
}
