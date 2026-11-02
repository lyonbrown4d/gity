import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

/** Right-hand attribute panel used on detail pages. */
export function PropertyPanel({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}): JSX.Element {
  return <aside className={cn("gity-property-panel", className)}>{children}</aside>;
}

export function PropertyItem({
  label,
  children,
  className,
}: {
  label: ReactNode;
  children: ReactNode;
  className?: string;
}): JSX.Element {
  return (
    <div className={cn("gity-property-item", className)}>
      <span className="gity-property-label">{label}</span>
      <div className="gity-property-value">{children}</div>
    </div>
  );
}
