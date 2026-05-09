import dayjs from "dayjs";
import relativeTime from "dayjs/plugin/relativeTime";
import type { RepositoryIssueCommentView, RepositoryIssueView } from "@/pages/types";

dayjs.extend(relativeTime);

export type IssueSortMode = "updated_desc" | "created_desc" | "number_desc" | "number_asc";

export interface IssueTimelineEvent {
  id: string;
  kind: "description" | "comment";
  authorUserId: string;
  content: string;
  createdAt: string;
  updatedAt: string;
}

export const extractErrorMessage = (error: unknown): string => {
  if (!(error instanceof Error)) {
    return "Unknown error";
  }
  const raw = error.message.trim();
  if (!raw) {
    return "Unknown error";
  }
  try {
    const parsed = JSON.parse(raw) as { message?: string };
    if (typeof parsed.message === "string" && parsed.message.trim().length > 0) {
      return parsed.message;
    }
  } catch {
    // ignore json parse failure
  }
  return raw;
};

export const toTimestamp = (value: string): number => {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return 0;
  }
  return date.getTime();
};

export const formatRelativeTime = (value: string): string => {
  const target = dayjs(value);
  if (!target.isValid()) {
    return value;
  }
  return target.fromNow();
};

export const issueUserInitials = (userId: string): string => {
  const value = userId.trim();
  if (!value) {
    return "U";
  }
  const chunks = value.split(/[_\-.@]+/g).filter(Boolean);
  if (chunks.length >= 2) {
    return `${chunks[0][0]}${chunks[1][0]}`.toUpperCase();
  }
  return value.slice(0, 2).toUpperCase();
};

export const filterAndSortIssues = (
  items: RepositoryIssueView[],
  status: "open" | "closed" | "all",
  query: string,
  sort: IssueSortMode,
): RepositoryIssueView[] => {
  const normalizedQuery = query.trim().toLowerCase();
  const filtered = items.filter((item) => {
    if (status !== "all" && item.status !== status) {
      return false;
    }
    if (!normalizedQuery) {
      return true;
    }
    return [
      `#${item.number}`,
      item.title,
      item.description ?? "",
      item.author_user_id,
      item.assignee_user_id ?? "",
    ].some((value) => value.toLowerCase().includes(normalizedQuery));
  });

  const sorted = [...filtered];
  sorted.sort((a, b) => {
    if (sort === "updated_desc") {
      return toTimestamp(b.updated_at) - toTimestamp(a.updated_at);
    }
    if (sort === "created_desc") {
      return toTimestamp(b.created_at) - toTimestamp(a.created_at);
    }
    if (sort === "number_asc") {
      return a.number - b.number;
    }
    return b.number - a.number;
  });
  return sorted;
};

export const buildIssueTimeline = (
  issue: RepositoryIssueView | null,
  comments: RepositoryIssueCommentView[],
): IssueTimelineEvent[] => {
  if (!issue) {
    return [];
  }
  const events: IssueTimelineEvent[] = [];
  if (issue.description && issue.description.trim().length > 0) {
    events.push({
      id: `${issue.id}-description`,
      kind: "description",
      authorUserId: issue.author_user_id,
      content: issue.description,
      createdAt: issue.created_at,
      updatedAt: issue.updated_at,
    });
  }
  for (const comment of comments) {
    events.push({
      id: comment.id,
      kind: "comment",
      authorUserId: comment.author_user_id,
      content: comment.content,
      createdAt: comment.created_at,
      updatedAt: comment.updated_at,
    });
  }
  events.sort((a, b) => toTimestamp(a.createdAt) - toTimestamp(b.createdAt));
  return events;
};
