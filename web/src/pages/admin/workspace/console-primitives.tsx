import type { ReactNode } from "react";
import { CircleDotDashed } from "lucide-react";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";

interface ConsolePageHeaderProps {
  title: ReactNode;
  description: ReactNode;
  eyebrow?: ReactNode;
  actions?: ReactNode;
}

export const ConsolePageHeader = ({
  title,
  description,
  eyebrow,
  actions,
}: ConsolePageHeaderProps): JSX.Element => <header className="flex flex-col gap-3 border-b border-border/80 pb-4 sm:flex-row sm:items-end sm:justify-between">
      <div className="min-w-0">
        {eyebrow ? (
          <p className="mb-1 font-mono text-[10px] font-semibold uppercase tracking-[0.18em] text-muted-foreground">
            {eyebrow}
          </p>
        ) : null}
        <h1 className="text-xl font-semibold tracking-tight text-foreground">{title}</h1>
        <p className="mt-1 max-w-3xl text-xs leading-5 text-muted-foreground">{description}</p>
      </div>
      {actions ? <div className="flex shrink-0 flex-wrap items-center gap-2">{actions}</div> : null}
    </header>;

interface ConsolePanelProps {
  title: ReactNode;
  description?: ReactNode;
  meta?: ReactNode;
  children: ReactNode;
  className?: string;
  bodyClassName?: string;
}

export const ConsolePanel = ({
  title,
  description,
  meta,
  children,
  className,
  bodyClassName,
}: ConsolePanelProps): JSX.Element => <section className={cn("overflow-hidden rounded-md border border-border/80 bg-card", className)}>
      <div className="flex min-h-12 items-center justify-between gap-3 border-b border-border/70 bg-muted/25 px-3 py-2">
        <div className="min-w-0">
          <h2 className="text-sm font-semibold">{title}</h2>
          {description ? <p className="mt-0.5 text-[11px] text-muted-foreground">{description}</p> : null}
        </div>
        {meta ? <div className="shrink-0 font-mono text-[10px] text-muted-foreground">{meta}</div> : null}
      </div>
      <div className={cn("p-3", bodyClassName)}>{children}</div>
    </section>;

interface MetricItem {
  label: ReactNode;
  value: ReactNode;
  detail: ReactNode;
  active?: boolean;
}

export const MetricStrip = ({ items }: { items: MetricItem[] }): JSX.Element => <dl className="grid overflow-hidden rounded-md border border-border/80 bg-card sm:grid-cols-3">
      {items.map((item, index) => (
        <div
          key={index}
          className="relative min-w-0 border-b border-border/70 px-4 py-3 last:border-b-0 sm:border-b-0 sm:border-r sm:last:border-r-0"
        >
          <dt className="flex items-center gap-2 text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
            <span className={cn("size-1.5 rounded-full bg-muted-foreground/50", item.active && "bg-primary")} />
            {item.label}
          </dt>
          <dd className="mt-2 flex items-baseline gap-2">
            <span className="font-mono text-2xl font-semibold tabular-nums text-foreground">{item.value}</span>
            <span className="truncate text-[11px] text-muted-foreground">{item.detail}</span>
          </dd>
        </div>
      ))}
    </dl>;

export const ConsoleLoadingRows = ({ rows = 3 }: { rows?: number }): JSX.Element => <div className="space-y-2" aria-label="Loading">
      {Array.from({ length: rows }, (_, index) => (
        <Skeleton key={index} className="h-9 w-full rounded-sm" />
      ))}
    </div>;

export const ConsoleEmpty = ({ children }: { children: ReactNode }): JSX.Element => <div className="flex min-h-28 flex-col items-center justify-center gap-2 px-4 text-center text-xs text-muted-foreground">
      <CircleDotDashed className="size-5" aria-hidden="true" />
      <p>{children}</p>
    </div>;

export const StatusDot = ({ active = true }: { active?: boolean }): JSX.Element => <span
      className={cn("inline-block size-1.5 rounded-full bg-muted-foreground/50", active && "bg-primary")}
      aria-hidden="true"
    />;
