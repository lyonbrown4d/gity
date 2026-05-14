import { useEffect, useMemo } from "react";
import { RefreshCw, ScrollText } from "lucide-react";
import { useCustom } from "@refinedev/core";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import type { RepositoryAuditEventView } from "@/pages/types";
import { extractErrorMessage, formatRelativeTime } from "./issues-utils";
import {
  isRecord,
  normalizeOptionalString,
  normalizeString,
  resolveBody,
  resolveRecordArray,
  type RawRecord,
} from "./repository-normalizers";

interface RepositoryAuditTabProps {
  repoId: string;
  t: (text: string) => string;
  onError: (message: string | null) => void;
}

export const RepositoryAuditTab = ({ repoId, t, onError }: RepositoryAuditTabProps): JSX.Element => {
  const auditQuery = useCustom<RawRecord[]>({
    url: `/projects/${repoId}/audit-events`,
    method: "get",
    config: {
      query: {
        limit: 100,
      },
    },
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });

  const events = useMemo(
    () => resolveAuditEvents(auditQuery.data?.data).map(normalizeAuditEvent),
    [auditQuery.data?.data],
  );
  const isLoading = auditQuery.isFetching && !auditQuery.data;

  const loadEvents = async () => {
    const result = await auditQuery.refetch();
    if (result.error) {
      onError(extractErrorMessage(result.error));
      return;
    }
    onError(null);
  };

  useEffect(() => {
    if (!auditQuery.error) {
      return;
    }
    onError(extractErrorMessage(auditQuery.error));
  }, [auditQuery.error, onError]);

  return (
    <Card className="card-enter">
      <CardHeader>
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <CardTitle>{t("Audit")}</CardTitle>
            <CardDescription>{t("Review project audit events emitted by repository, CI, and collaboration workflows.")}</CardDescription>
          </div>
          <Button type="button" size="sm" variant="ghost" onClick={() => void loadEvents()}>
            <RefreshCw className="size-4" />
            {t("Reload")}
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="rounded-md border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t("Event")}</TableHead>
                <TableHead>{t("Actor")}</TableHead>
                <TableHead>{t("Target")}</TableHead>
                <TableHead>{t("Summary")}</TableHead>
                <TableHead>{t("Time")}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow>
                  <TableCell colSpan={5} className="text-sm text-muted-foreground">
                    {t("Loading audit events...")}
                  </TableCell>
                </TableRow>
              ) : null}
              {!isLoading && events.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={5} className="text-sm text-muted-foreground">
                    {t("No audit events found.")}
                  </TableCell>
                </TableRow>
              ) : null}
              {events.map((event) => (
                <TableRow key={event.id}>
                  <TableCell>
                    <div className="flex flex-wrap items-center gap-2">
                      <ScrollText className="size-4 text-muted-foreground" />
                      <span className="font-medium">{event.event_name || event.action}</span>
                      {event.action ? <Badge variant="outline">{event.action}</Badge> : null}
                    </div>
                  </TableCell>
                  <TableCell className="font-mono text-xs">#{event.actor_user_id || "system"}</TableCell>
                  <TableCell className="text-xs text-muted-foreground">
                    {event.target_type || t("N/A")} {event.target_id ? `#${event.target_id}` : ""}
                  </TableCell>
                  <TableCell className="max-w-[360px] truncate">{event.summary || event.payload || t("N/A")}</TableCell>
                  <TableCell className="text-xs text-muted-foreground">
                    {event.created_at ? formatRelativeTime(event.created_at) : t("N/A")}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>
  );
};

const resolveAuditEvents = (payload: unknown): RawRecord[] => {
  if (Array.isArray(payload)) {
    return payload.filter(isRecord);
  }
  return resolveRecordArray(resolveBody(payload));
};

const normalizeAuditEvent = (raw: RawRecord): RepositoryAuditEventView => ({
  id: normalizeString(raw.id ?? raw.ID),
  project_id: normalizeString(raw.project_id ?? raw.ProjectID),
  organization_id: normalizeString(raw.organization_id ?? raw.OrganizationID),
  event_name: normalizeString(raw.event_name ?? raw.EventName),
  action: normalizeString(raw.action ?? raw.Action),
  actor_user_id: normalizeString(raw.actor_user_id ?? raw.ActorUserID),
  target_type: normalizeString(raw.target_type ?? raw.TargetType),
  target_id: normalizeString(raw.target_id ?? raw.TargetID),
  summary: normalizeString(raw.summary ?? raw.Summary),
  payload: normalizeOptionalString(raw.payload ?? raw.Payload),
  created_at: normalizeString(raw.created_at ?? raw.CreatedAt),
});
