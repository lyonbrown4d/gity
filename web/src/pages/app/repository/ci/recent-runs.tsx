import { useState } from "react";
import type { CiPipeline } from "@/lib/generated/model/ciPipeline";
import { CIStatus, elapsed } from "./status";
interface Props {
  pipelines: CiPipeline[];
  selected?: string;
  onSelect: (id: string) => void;
  t: (key: string) => string;
}
export const RecentRuns = ({ pipelines, selected, onSelect, t }: Props) => {
  const [filter, setFilter] = useState("all");
  const rows = pipelines.filter((pipeline) => filter === "all" || pipeline.status === filter);
  return (
    <section className="space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold">{t("Recent runs")}</h3>
        <select
          className="rounded-md border border-input bg-card p-2 text-sm"
          aria-label={t("Status")}
          value={filter}
          onChange={(event) => setFilter(event.target.value)}
        >
          {["all", "pending", "running", "succeeded", "failed", "canceled"].map((status) => (
            <option key={status} value={status}>
              {t(status === "all" ? "All statuses" : status)}
            </option>
          ))}
        </select>
      </div>
      <div className="overflow-x-auto rounded-lg border border-border bg-card">
        <table className="w-full text-left text-sm">
          <thead className="bg-muted/50 text-xs text-muted-foreground">
            <tr>
              {["Pipeline", "Status", "Source", "Branch", "Commit", "Created", "Duration"].map((label) => (
                <th className="px-4 py-3 font-medium" key={label}>
                  {t(label)}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.map((pipeline) => (
              <tr
                className={`border-t border-border hover:bg-muted/30 ${pipeline.id === selected ? "bg-primary/5" : ""}`}
                key={pipeline.id}
              >
                <td className="px-4 py-3">
                  <button className="font-semibold text-primary hover:underline" onClick={() => onSelect(pipeline.id)}>
                    #{pipeline.iid}
                  </button>
                </td>
                <td className="px-4">
                  <CIStatus status={pipeline.status} t={t} />
                </td>
                <td className="px-4">{t(pipeline.source)}</td>
                <td className="px-4">{pipeline.ref_name}</td>
                <td className="px-4 font-mono text-xs">{pipeline.commit_sha.slice(0, 12)}</td>
                <td className="whitespace-nowrap px-4 text-xs">
                  {pipeline.created_at ? new Date(pipeline.created_at).toLocaleString() : "—"}
                </td>
                <td className="whitespace-nowrap px-4">{elapsed(pipeline.started_at, pipeline.finished_at)}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {!rows.length && <p className="p-5 text-sm text-muted-foreground">{t("No pipelines match this status.")}</p>}
      </div>
    </section>
  );
};
