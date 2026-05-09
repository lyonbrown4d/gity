import type { ReactNode } from "react";
import type { LucideIcon } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";

interface ProductHeroProps {
  eyebrow?: ReactNode;
  title: ReactNode;
  description?: ReactNode;
  actions?: ReactNode;
  aside?: ReactNode;
  children?: ReactNode;
  className?: string;
  contentClassName?: string;
}

export function ProductHero({
  eyebrow,
  title,
  description,
  actions,
  aside,
  children,
  className,
  contentClassName,
}: ProductHeroProps): JSX.Element {
  return (
    <Card className={cn("gity-hero", className)}>
      <CardHeader className={cn("relative z-10", children ? "pb-4" : undefined)}>
        <div className="grid gap-6 lg:grid-cols-[1.45fr_0.85fr] lg:items-end">
          <div className="max-w-3xl space-y-5">
            {eyebrow ? <ProductEyebrow>{eyebrow}</ProductEyebrow> : null}
            <div className="space-y-3">
              <CardTitle className="gity-hero-title">{title}</CardTitle>
              {description ? (
                <CardDescription className="max-w-2xl text-sm leading-6">{description}</CardDescription>
              ) : null}
            </div>
            {actions ? <div className="flex flex-wrap gap-2">{actions}</div> : null}
          </div>
          {aside ? <div>{aside}</div> : null}
        </div>
      </CardHeader>
      {children ? (
        <CardContent className={cn("relative z-10", contentClassName)}>
          {children}
        </CardContent>
      ) : null}
    </Card>
  );
}

export function ProductEyebrow({ children, className }: { children: ReactNode; className?: string }): JSX.Element {
  return (
    <div className={cn("inline-flex items-center gap-2 rounded-full border bg-background/65 px-3 py-1 text-xs font-semibold text-muted-foreground", className)}>
      <StatusDot />
      {children}
    </div>
  );
}

export function StatusDot({ className }: { className?: string }): JSX.Element {
  return <span className={cn("gity-dot", className)} />;
}

export function ProductMetricCard({
  icon: Icon,
  label,
  value,
  description,
  className,
}: {
  icon: LucideIcon;
  label: ReactNode;
  value: ReactNode;
  description?: ReactNode;
  className?: string;
}): JSX.Element {
  return (
    <div className={cn("gity-metric-card card-enter", className)}>
      <div className="flex items-start justify-between gap-4">
        <div>
          <p className="text-sm text-muted-foreground">{label}</p>
          <p className="mt-2 text-4xl font-semibold tracking-[-0.05em]">{value}</p>
        </div>
        <div className="flex size-11 items-center justify-center rounded-2xl bg-primary/10 text-primary">
          <Icon className="size-5" />
        </div>
      </div>
      {description ? <p className="mt-5 text-xs leading-5 text-muted-foreground">{description}</p> : null}
    </div>
  );
}

export function ProductFeatureList({
  items,
  className,
}: {
  items: Array<{ icon: LucideIcon; text: ReactNode }>;
  className?: string;
}): JSX.Element {
  return (
    <div className={cn("grid gap-3", className)}>
      {items.map((item, index) => (
        <div key={index} className="flex items-center gap-3 rounded-2xl border border-background/15 bg-background/10 p-3 backdrop-blur">
          <item.icon className="size-4 text-primary" />
          <span className="text-sm text-background/80">{item.text}</span>
        </div>
      ))}
    </div>
  );
}

export function ProductCodePanel({
  title,
  children,
  className,
}: {
  title?: ReactNode;
  children: ReactNode;
  className?: string;
}): JSX.Element {
  return (
    <div className={cn("gity-code-panel", className)}>
      <div className="mb-3 flex items-center gap-2 text-background/55">
        <span className="size-2 rounded-full bg-red-400" />
        <span className="size-2 rounded-full bg-amber-300" />
        <span className="size-2 rounded-full bg-emerald-300" />
        {title ? <span className="ml-2">{title}</span> : null}
      </div>
      <pre className="whitespace-pre-wrap leading-6">{children}</pre>
    </div>
  );
}

export function ProductStatusBadge({
  icon: Icon,
  children,
  variant = "outline",
  className,
}: {
  icon?: LucideIcon;
  children: ReactNode;
  variant?: "default" | "secondary" | "outline";
  className?: string;
}): JSX.Element {
  return (
    <Badge variant={variant} className={cn("gap-1", className)}>
      {Icon ? <Icon className="size-3" /> : null}
      {children}
    </Badge>
  );
}
