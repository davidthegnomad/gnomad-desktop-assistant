import { openUrl } from "@tauri-apps/plugin-opener";
import { STUDIO_LABEL, STUDIO_URL } from "../lib/brand";

interface StudioLinkProps {
  className?: string;
}

/** Gnomad Studio mark — links to gnomadstudio.org */
export function StudioLink({ className = "" }: StudioLinkProps) {
  return (
    <button
      type="button"
      className={`studio-link ${className}`.trim()}
      onClick={() => void openUrl(STUDIO_URL)}
      title="Gnomad Studio"
      aria-label="Gnomad Studio — gnomadstudio.org"
    >
      {STUDIO_LABEL}
    </button>
  );
}
