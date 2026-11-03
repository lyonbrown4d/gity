import { Code2, CircleDot, type LucideIcon } from "lucide-react";
import type { SearchHit } from "@/lib/generated/model";

export interface GlobalSearchQuickJumpItem {
  id: string;
  title: string;
  description: string;
  path: string;
  icon: LucideIcon;
  keywords: string[];
}

export const toGlobalSearchQuickJumpItem = (
  hit: SearchHit,
): GlobalSearchQuickJumpItem | null => {
  const base = `/app/projects/${encodeURIComponent(hit.organization_id)}/${encodeURIComponent(hit.project_id)}`;
  if (hit.kind === "issue" && hit.issue_iid) {
    return {
      id: `global-issue:${hit.project_id}:${hit.issue_iid}`,
      title: hit.title,
      description: `${hit.project_path} · #${hit.issue_iid}${hit.state ? ` · ${hit.state}` : ""} · ${hit.snippet}`,
      path: `${base}/issues/${encodeURIComponent(hit.issue_iid)}`,
      icon: CircleDot,
      keywords: [hit.project_path, hit.issue_iid, hit.snippet],
    };
  }
  if (hit.kind === "code" && hit.path && hit.commit_sha && hit.line !== null && hit.line !== undefined) {
    const params = new URLSearchParams({ tab: "code", commit_sha: hit.commit_sha, path: hit.path, line: String(hit.line) });
    return {
      id: `global-code:${hit.project_id}:${hit.commit_sha}:${hit.path}:${hit.line}`,
      title: hit.path,
      description: `${hit.project_path} · ${hit.commit_sha.slice(0, 12)}:${hit.line} · ${hit.snippet}`,
      path: `${base}?${params.toString()}#L${hit.line}`,
      icon: Code2,
      keywords: [hit.project_path, hit.commit_sha, hit.snippet],
    };
  }
  return null;
};
