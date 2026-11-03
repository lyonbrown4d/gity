import type {
  RepositoryPackageDetailView,
  RepositoryPackageFileContentView,
  RepositoryPackageFileView,
  RepositoryPackageVersionDetailView,
  RepositoryPackageVersionView,
  RepositoryPackageView,
} from "@/pages/types";
import {
  isRecord,
  normalizeNumber,
  normalizeOptionalString,
  normalizeString,
  resolveArrayPayload,
  resolveBody,
  resolveRecordArray,
  type RawRecord,
} from "../repository-normalizers";
type RawPackage = RawRecord;

export const resolvePackageList = (payload: unknown): RawPackage[] => {
  const raw = resolveBody(payload);
  // The Rust backend returns a page envelope ({items,page,per_page,total,total_pages}).
  return resolveArrayPayload<RawPackage>(raw);
};

export const normalizePackageDetail = (payload: unknown): RepositoryPackageDetailView | null => {
  const raw = resolveBody(payload);
  if (!isRecord(raw)) {
    return null;
  }
  const versionsRaw = raw.versions ?? raw.Versions;
  return {
    package: normalizePackage(raw.package ?? raw.Package),
    versions: resolveRecordArray(versionsRaw).map(normalizePackageVersionDetail),
  };
};

export const normalizePackageVersionDetail = (raw: RawPackage): RepositoryPackageVersionDetailView => {
  const files = raw.files ?? raw.Files;
  return {
    version: normalizePackageVersion(raw.version ?? raw.Version),
    files: resolveRecordArray(files).map(normalizePackageFile),
  };
};

export const normalizePackageFileContent = (payload: unknown): RepositoryPackageFileContentView => {
  const raw = resolveBody(payload);
  if (!isRecord(raw)) {
    return { file: normalizePackageFile({}), content_base64: "" };
  }
  return {
    file: normalizePackageFile(raw.file ?? raw.File),
    content_base64: normalizeString(raw.content_base64 ?? raw.ContentBase64 ?? raw.Content),
  };
};

export const normalizePackage = (value: unknown): RepositoryPackageView => {
  const raw = isRecord(value) ? value : {};
  return {
    id: normalizeString(raw.id ?? raw.ID),
    project_id: normalizeString(raw.project_id ?? raw.ProjectID),
    type: normalizeString(raw.type ?? raw.Type) || "generic",
    name: normalizeString(raw.name ?? raw.Name) || "package",
    created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
    updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
  };
};

export const normalizePackageVersion = (value: unknown): RepositoryPackageVersionView => {
  const raw = isRecord(value) ? value : {};
  return {
    id: normalizeString(raw.id ?? raw.ID),
    project_package_id: normalizeString(raw.project_package_id ?? raw.ProjectPackageID),
    version: normalizeString(raw.version ?? raw.Version) || "0.0.0",
    status: normalizeString(raw.status ?? raw.Status) || "default",
    created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
    updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
  };
};

export const normalizePackageFile = (value: unknown): RepositoryPackageFileView => {
  const raw = isRecord(value) ? value : {};
  return {
    id: normalizeString(raw.id ?? raw.ID),
    project_package_version_id: normalizeString(raw.project_package_version_id ?? raw.ProjectPackageVersionID),
    file_name: normalizeString(raw.file_name ?? raw.FileName) || "artifact",
    file_path: normalizeOptionalString(raw.file_path ?? raw.FilePath),
    content_type: normalizeOptionalString(raw.content_type ?? raw.ContentType),
    byte_size: normalizeNumber(raw.byte_size ?? raw.ByteSize),
    created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
    updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
  };
};
