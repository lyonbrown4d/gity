import type { ComponentType } from "react";
import { Card } from "@/components/ui/card";
import { cn } from "@/lib/utils";

interface StatCardProps {
  icon: ComponentType<{ className?: string }>;
  label: string;
  value: string;
  detail?: string;
  className?: string;
}

/**
 * Compact metric card. Cards are reserved for standalone, emphasised content
 * such as dashboard metrics; page structure uses `Section` instead.
 */
export function StatCard({ icon: Icon, label, value, detail, className }: StatCardProps): JSX.Element {
  return (
    <Card className={cn("flex items-center gap-3 rounded-md border-border p-3 shadow-none", className)}>
      <span className="flex size-8 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground">
        <Icon className="size-4" />
      </span>
      <div className="min-w-0">
        <p className="truncate text-xs text-muted-foreground">{label}</p>
        <p className="truncate text-sm font-semibold">{value}</p>
        {detail && (
          <p className="mt-0.5 truncate text-[11px] text-muted-foreground" title={detail}>
            {detail}
          </p>
        )}
      </div>
    </Card>
  );
}
