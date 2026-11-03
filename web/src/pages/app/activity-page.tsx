import { RefreshCw } from "lucide-react";
import { ActivityFeed } from "@/components/activity/activity-feed";
import { Section } from "@/components/surface/section";
import { Button } from "@/components/ui/button";
import { useI18n } from "@/lib/i18n";
import { useActivity } from "./use-activity";

const ACTIVITY_LIMIT = 50;

export const AppActivityPage = (): JSX.Element => {
  const { t } = useI18n();
  const activity = useActivity(ACTIVITY_LIMIT);
  const queryError = activity.query.error instanceof Error ? activity.query.error.message : null;
  const isRefreshing = activity.query.isFetching;

  return (
    <div className="flex flex-col gap-4 page-enter">
      <section aria-labelledby="activity-title" className="overflow-hidden rounded-md border border-border bg-card">
        <div className="flex flex-col gap-3 px-4 py-4 sm:flex-row sm:items-center sm:justify-between">
          <div className="min-w-0">
            <div className="mb-1 flex items-center gap-2 text-xs font-medium text-muted-foreground">
              <span className="size-2 rounded-full bg-primary" aria-hidden="true" />
              {t("Workspace")} / {t("Activity")}
            </div>
            <h1 id="activity-title" className="text-lg font-semibold tracking-tight">{t("Activity")}</h1>
            <p className="mt-1 max-w-2xl text-sm text-muted-foreground">
              {t("Events across the projects you can access, newest first.")}
            </p>
          </div>
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => void activity.query.refetch()}
            disabled={isRefreshing}
          >
            <RefreshCw data-icon="inline-start" />
            {isRefreshing ? t("Refreshing...") : t("Refresh")}
          </Button>
        </div>
      </section>

      <Section
        title={t("Latest activity")}
        description={t("Showing the 50 most recent events across the projects you can access.")}
        flush
      >
        <ActivityFeed
          items={activity.items}
          isLoading={activity.query.isLoading}
          error={queryError}
          emptyText={t("No recent activity yet.")}
          skeletonRows={8}
        />
      </Section>
    </div>
  );
};
