import { CheckCircle2, Clock3, ShieldCheck, XCircle } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import type { RepositoryMergeRequestCheckStatusView, RepositoryMergeRequestState } from "@/pages/types";

export const MergeRequestStat = ({ label, value, tone }: {
  label: string;
  value: number;
  tone: "success" | "primary" | "muted";
}): JSX.Element => (
  <div className="flex items-center gap-2 px-4 py-2">
    <span className={statTone[tone]} aria-hidden="true" />
    <span className="text-xs text-muted-foreground">{label}</span>
    <span className="ml-auto text-sm font-semibold tabular-nums">{value}</span>
  </div>
);

const statTone: Record<"success" | "primary" | "muted", string> = {
  success: "size-1.5 shrink-0 rounded-full bg-success",
  primary: "size-1.5 shrink-0 rounded-full bg-primary",
  muted: "size-1.5 shrink-0 rounded-full bg-muted-foreground",
};

export const MergeRequestStateBadge = ({ state, t }: {
  state: RepositoryMergeRequestState;
  t: (text: string) => string;
}): JSX.Element => {
  const variant = state === "opened" ? "default" : state === "merged" ? "secondary" : "outline";
  const label = state === "opened" ? "Open" : state === "merged" ? "Merged" : "Closed";
  return <Badge variant={variant}>{t(label)}</Badge>;
};

export const CheckLine = ({ label, value }: { label: string; value: string }): JSX.Element => (
  <div className="flex items-baseline justify-between gap-3">
    <span className="shrink-0 text-muted-foreground">{label}</span>
    <span className="truncate font-medium text-foreground">{value}</span>
  </div>
);

export const checkStatusMeta = (checks: RepositoryMergeRequestCheckStatusView) => {
  if (!checks.required) return { label: "No checks required", Icon: ShieldCheck };
  if (checks.mergeable) return { label: "Ready to merge", Icon: CheckCircle2 };
  if (checks.status === "missing") return { label: "Pipeline missing", Icon: Clock3 };
  return { label: "Blocked", Icon: XCircle };
};

export const shortText = (value: string, length = 8): string => (
  value.length > length ? value.slice(0, length) : value
);
