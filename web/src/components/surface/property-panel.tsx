import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

/** Right-hand attribute panel used on detail pages. */
export const PropertyPanel = ({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}): JSX.Element => <aside className={cn("gity-property-panel", className)}>{children}</aside>;

export const PropertyItem = ({
  label,
  children,
  className,
}: {
  label: ReactNode;
  children: ReactNode;
  className?: string;
}): JSX.Element => <div className={cn("gity-property-item", className)}>
      <span className="gity-property-label">{label}</span>
      <div className="gity-property-value">{children}</div>
    </div>;
