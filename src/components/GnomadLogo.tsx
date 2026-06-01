import { MUSHROOM } from "../lib/brand";

type Size = "sm" | "md" | "lg";

const sizes: Record<Size, string> = {
  sm: "1.1rem",
  md: "1.45rem",
  lg: "2rem",
};

interface GnomadLogoProps {
  size?: Size;
  className?: string;
}

/** Brand mark: mushroom */
export function GnomadLogo({ size = "md", className = "" }: GnomadLogoProps) {
  return (
    <span
      className={`gnomad-logo ${className}`.trim()}
      style={{ fontSize: sizes[size] }}
      aria-hidden
    >
      {MUSHROOM}
    </span>
  );
}
