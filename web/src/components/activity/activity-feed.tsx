import type { ComponentType } from "react";
import { useNavigate } from "react-router";
import { Activity, CircleDot, GitCommitHorizontal, GitPullRequest, Package } from "lucide-react";
import { DataList, DataRow } from "@/components/surface/data-list";
import { Skeleton } from "@/components/ui/skeleton";
import { formatRelativeTime } from "@/pages/app/repository/issues-utils";

export interface ActivityItem {
  id: string;
  eventName: string;
  action: string;
  actorName: string;
  projectId: string;
  projectName: string;
  organizationId: string;
  projectPath: string;
  targetType: string;
  targetId: string;
  summary: string;
  createdAt: string;
}

type ActivityIconComponent = ComponentType<{ className?: string }>;

const eventIcons: Array<{ prefix: string; icon: ActivityIconComponent }> = [
  { prefix: "push", icon: GitCommitHorizontal },
  { prefix: "issue", icon: CircleDot },
  { prefix: "merge_request", icon: GitPullRequest },
  { prefix: "package", icon: Package },
];

/** Picks the row icon from the `event_name` prefix, falling back to a generic event. */
export function activityIcon(eventName: string): ActivityIconComponent {
  const name = eventName.trim().toLowerCase();
  return eventIcons.find((entry) => name.startsWith(entry.prefix))?.icon ?? Activity;
}

/** Normalizes the `/activity` payload; the Orval client can replace this once generated. */
export function toActivityItems(payload: unknown): ActivityItem[] {
  const rows = activityRows(payload);
  return rows
    .map((row) => normalizeActivityItem(row))
    .filter((item) => item.id !== "" || item.summary !== "" || item.eventName !== "");
}

function activityRows(payload: unknown): Record<string, unknown>[] {
  if (Array.isArray(payload)) {
    return payload.filter(isRecord);
  }
  if (!isRecord(payload)) {
    return [];
  }
  const nested = payload.body ?? payload.Body;
  if (nested !== undefined) {
    const rows = activityRows(nested);
    if (rows.length > 0) {
      return rows;
    }
  }
  const items = payload.items ?? payload.Items;
  return Array.isArray(items) ? items.filter(isRecord) : [];
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function text(value: unknown): string {
  if (typeof value === "string") {
    return value.trim();
  }
  if (typeof value === "number") {
    return String(value);
  }
  return "";
}

function normalizeActivityItem(row: Record<string, unknown>): ActivityItem {
  const actorId = text(row.actor_user_id);
  const eventName = text(row.event_name);
  return {
    id: text(row.id),
    eventName,
    action: text(row.action),
    actorName: text(row.actor_name) || (actorId ? `#${actorId}` : "system"),
    projectId: text(row.project_id),
    projectName: text(row.project_name) || "—",
    organizationId: text(row.organization_id),
    projectPath: text(row.project_path),
    targetType: text(row.target_type),
    targetId: text(row.target_id),
    summary: text(row.summary) || text(row.action) || eventName,
    createdAt: text(row.created_at),
  };
}

interface ActivityFeedProps {
  items: ActivityItem[];
  emptyText: string;
  isLoading?: boolean;
  error?: string | null;
  skeletonRows?: number;
}

/** Divided activity rows shared by the dashboard preview and the activity page. */
export function ActivityFeed({
  items,
  emptyText,
  isLoading = false,
  error = null,
  skeletonRows = 4,
}: ActivityFeedProps): JSX.Element {
  const navigate = useNavigate();
  if (isLoading) {
    return (
      <div className="divide-y divide-border" aria-label={emptyText}>
        {Array.from({ length: skeletonRows }, (_, index) => (
          <div key={index} className="px-4 py-2.5">
            <Skeleton className="h-8 w-full" />
          </div>
        ))}
      </div>
    );
  }
  if (error) {
    return <p className="px-4 py-6 text-sm text-muted-foreground">{error}</p>;
  }
  if (items.length === 0) {
    return <p className="px-4 py-6 text-sm text-muted-foreground">{emptyText}</p>;
  }
  return (
    <DataList>
      {items.map((item) => {
        const Icon = activityIcon(item.eventName);
        return (
          <DataRow
            key={item.id || `${item.eventName}-${item.createdAt}-${item.summary}`}
            onClick={
              item.organizationId && item.projectId
                ? () => navigate(`/app/projects/${item.organizationId}/${item.projectId}`)
                : undefined
            }
          >
            <span className="flex size-7 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground">
              <Icon className="size-3.5" aria-hidden="true" />
            </span>
            <div className="min-w-0 flex-1">
              <p className="truncate text-[13px] font-medium">{item.summary}</p>
              <p className="mt-0.5 flex min-w-0 items-center gap-1.5 truncate text-[11px] text-muted-foreground">
                <span className="truncate">{item.actorName}</span>
                <span aria-hidden="true">·</span>
                <span className="truncate">{item.projectName}</span>
              </p>
            </div>
            {item.createdAt ? (
              <time className="shrink-0 text-[11px] text-muted-foreground" dateTime={item.createdAt}>
                {formatRelativeTime(item.createdAt)}
              </time>
            ) : null}
          </DataRow>
        );
      })}
    </DataList>
  );
}
