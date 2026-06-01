/** Soft cursor bloom — small colorful core, no visible outer ring. */
export function CursorBloomBackground() {
  return (
    <div className="gemini-gradient-bg" aria-hidden>
      <div className="cursor-bloom cursor-bloom-mesh" />
      <div className="cursor-bloom cursor-bloom-core" />
      <div className="cursor-bloom cursor-bloom-tint" />
    </div>
  );
}
