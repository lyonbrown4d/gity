import { Ban, CheckCircle2, Clock3, Loader2, XCircle } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import "./flow.scss";
export const isTerminal = (status: string) => ["succeeded", "failed", "canceled", "cancelled"].includes(status);
export const CIStatus = ({ status, t }: { status: string; t: (key: string) => string }) => {
  const Icon =
    status === "succeeded"
      ? CheckCircle2
      : status === "failed"
        ? XCircle
        : status === "running"
          ? Loader2
          : status === "canceled" || status === "cancelled"
            ? Ban
            : Clock3;
  return (
    <Badge
      variant="outline"
      className={`gap-1.5 whitespace-nowrap ${status === "failed" ? "border-destructive/25 bg-destructive/10 text-destructive" : status === "succeeded" ? "ci-status-success" : "text-muted-foreground"}`}
    >
      <Icon className={`size-3.5 ${status === "running" ? "animate-spin motion-reduce:animate-none" : ""}`} />
      {t(status)}
    </Badge>
  );
};
export const elapsed = (start?: string | null, finish?: string | null): string => {
  if (!start || !finish) return "—";
  const seconds = Math.max(0, Math.floor((Date.parse(finish) - Date.parse(start)) / 1000));
  return Number.isFinite(seconds) ? `${Math.floor(seconds / 60)}m ${seconds % 60}s` : "—";
};
export const saveBlob = (blob: Blob, filename: string) => {
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
};
