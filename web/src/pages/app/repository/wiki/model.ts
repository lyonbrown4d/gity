import type { RepositoryWikiPageView } from "@/pages/types";
import {
  isRecord,
  normalizeOptionalString,
  normalizeString,
  resolveRecordArray,
  type RawRecord,
} from "../repository-normalizers";

export type RawWikiPage = RawRecord;

export const resolveWikiList = (payload: unknown): RawWikiPage[] => {
  if (Array.isArray(payload)) {
    return resolveRecordArray(payload);
  }
  return isRecord(payload) ? resolveRecordArray(payload.body ?? payload.Body) : [];
};

export const resolveWikiPage = (payload: unknown): RawWikiPage => {
  if (isRecord(payload)) {
    const nested = payload.body ?? payload.Body;
    return isRecord(nested) ? nested : payload;
  }
  return {};
};

export const normalizeWikiPage = (raw: RawWikiPage): RepositoryWikiPageView => ({
  id: normalizeString(raw.id ?? raw.ID),
  project_id: normalizeString(raw.project_id ?? raw.ProjectID),
  slug: normalizeString(raw.slug ?? raw.Slug),
  title: normalizeString(raw.title ?? raw.Title) || "Untitled",
  content: normalizeString(raw.content ?? raw.Content),
  format: normalizeString(raw.format ?? raw.Format) || "markdown",
  author_user_id: normalizeString(raw.author_user_id ?? raw.AuthorUserID),
  last_edited_by_user_id: normalizeString(raw.last_edited_by_user_id ?? raw.LastEditedByUserID),
  created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
  updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
});
