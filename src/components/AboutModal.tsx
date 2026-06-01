import { useRef } from "react";
import { GnomadLogo } from "./GnomadLogo";
import { APP_NAME, VERSION } from "../lib/brand";
import { StudioLink } from "./StudioLink";
import { useFocusTrap } from "../hooks/useFocusTrap";

interface AboutModalProps {
  onClose: () => void;
}

export function AboutModal({ onClose }: AboutModalProps) {
  const dialogRef = useRef<HTMLDivElement>(null);
  useFocusTrap(dialogRef, true);

  return (
    <div className="modal-overlay" onClick={onClose} role="presentation">
      <div
        ref={dialogRef}
        className="onboarding-card about-card"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
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
