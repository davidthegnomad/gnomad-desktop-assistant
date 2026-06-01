import { GnomadLogo } from "./GnomadLogo";
import { APP_NAME, VERSION } from "../lib/brand";
import { StudioLink } from "./StudioLink";

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
        <p className="onboarding-subtitle about-credit">
          Built with ❤️ by <StudioLink className="about-studio-inline" />
        </p>
        <div className="onboarding-actions">
          <button type="button" className="btn primary" onClick={onClose}>
            OK
          </button>
        </div>
      </div>
    </div>
  );
}
