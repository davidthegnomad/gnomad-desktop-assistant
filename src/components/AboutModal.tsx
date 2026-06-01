import { openUrl } from "@tauri-apps/plugin-opener";
import { GnomadLogo } from "./GnomadLogo";
import {
  APP_NAME,
  ABOUT_CREDIT,
  STUDIO_URL,
  VERSION,
} from "../lib/brand";

interface AboutModalProps {
  onClose: () => void;
}

export function AboutModal({ onClose }: AboutModalProps) {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div
        className="onboarding-card about-card"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-labelledby="about-title"
      >
        <GnomadLogo size="lg" />
        <h2 id="about-title" className="onboarding-title">
          {APP_NAME}
        </h2>
        <p className="about-version">Version {VERSION}</p>
        <p className="onboarding-subtitle about-credit">{ABOUT_CREDIT}</p>
        <button
          type="button"
          className="about-link"
          onClick={() => openUrl(STUDIO_URL)}
        >
          🍄 gnomadstudio.org
        </button>
        <div className="onboarding-actions">
          <button type="button" className="btn primary" onClick={onClose}>
            OK
          </button>
        </div>
      </div>
    </div>
  );
}
