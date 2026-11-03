import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

interface SectionProps {
  title?: ReactNode;
  description?: ReactNode;
  actions?: ReactNode;
  children: ReactNode;
  className?: string;
  /** Remove body padding for flush lists. */
  flush?: boolean;
}

/** Flat panel that replaces the decorative Card on content pages. */
export const Section = ({
  title,
  description,
  actions,
  children,
  className,
  flush = false,
}: SectionProps): JSX.Element => {
  const hasHeader = Boolean(title || description || actions);
  return (
    <section className={cn("gity-section min-w-0", className)}>
      {hasHeader && (
        <header className="gity-section-header flex-wrap">
          <div className="min-w-0 flex-[1_1_12rem]">
            {title && <h2 className="gity-section-title">{title}</h2>}
            {description && <p className="gity-section-description">{description}</p>}
          </div>
          {actions && <div className="gity-section-actions max-w-full flex-wrap">{actions}</div>}
        </header>
      )}
      <div className={flush ? undefined : "gity-section-body"}>{children}</div>
    </section>
  );
};
