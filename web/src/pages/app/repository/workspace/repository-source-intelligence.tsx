import { Activity, BarChart3, RefreshCw } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { RepositoryLanguagesView } from "@/pages/types";
import { formatBytes, formatTime } from "../repository-utils";

interface RepositorySourceIntelligenceProps {
  t: (text: string) => string;
  branch: string;
  languages: RepositoryLanguagesView | null;
  isLoading: boolean;
  onRefresh: () => void;
}

export const RepositorySourceIntelligence = ({
  t, branch, languages, isLoading, onRefresh,
}: RepositorySourceIntelligenceProps): JSX.Element => (
  <aside className="min-w-0 bg-muted/10 xl:border-l xl:border-border">
    <div className="flex h-10 items-center justify-between border-b border-border px-3">
      <div className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
        <Activity className="size-3.5" />
        {t("Repository pulse")}
      </div>
      <Badge variant="outline" className="max-w-28 truncate font-mono text-[10px]">{branch || "--"}</Badge>
    </div>

    <section className="p-3">
      <div className="flex items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <BarChart3 className="size-4 text-muted-foreground" />
          <h2 className="text-sm font-medium">{t("Languages")}</h2>
        </div>
        <Button type="button" size="icon" variant="ghost" className="size-7" disabled={!branch || isLoading} onClick={onRefresh} aria-label={t("Refresh")}>
          <RefreshCw className={isLoading ? "size-3.5 animate-spin" : "size-3.5"} />
        </Button>
      </div>

      <div className="mt-4 space-y-3">
        {isLoading ? <p className="text-xs text-muted-foreground">{t("Loading language statistics...")}</p> : null}
        {!isLoading && languages?.status === "pending" ? (
          <p className="text-xs text-muted-foreground">{t("Language statistics are being computed...")}</p>
        ) : null}
        {!isLoading && languages?.status === "ready" && languages.languages.length > 0 ? (
          languages.languages.map((item) => (
            <div key={item.language}>
              <div className="mb-1 flex items-center justify-between gap-2 text-xs">
                <span className="truncate font-medium">{item.language}</span>
                <span className="font-mono text-[10px] text-muted-foreground">{item.percentage.toFixed(1)}%</span>
              </div>
              <div className="h-1 overflow-hidden rounded-full bg-muted">
                <div className="h-full rounded-full bg-primary" style={{ width: String(Math.max(item.percentage, item.percentage > 0 ? 2 : 0)) + "%" }} />
              </div>
            </div>
          ))
        ) : null}
        {!isLoading && (!languages || languages.languages.length === 0) && languages?.status !== "pending" ? (
          <p className="text-xs text-muted-foreground">{t("No language statistics yet.")}</p>
        ) : null}
      </div>
    </section>

    {languages?.status === "ready" ? (
      <dl className="grid grid-cols-2 border-t border-border text-xs">
        <div className="border-r border-border p-3">
          <dt className="text-muted-foreground">{t("Total size")}</dt>
          <dd className="mt-1 font-mono font-medium">{formatBytes(languages.total_bytes)}</dd>
        </div>
        <div className="p-3">
          <dt className="text-muted-foreground">{t("Last analyzed")}</dt>
          <dd className="mt-1 truncate font-mono text-[10px] font-medium">
            {languages.analyzed_at ? formatTime(languages.analyzed_at) : t("N/A")}
          </dd>
        </div>
      </dl>
    ) : null}
  </aside>
);
