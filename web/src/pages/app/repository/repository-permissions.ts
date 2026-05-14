import { isRecord, normalizeBoolean, resolveBody } from "./repository-normalizers";

export type RepositoryRole = "guest" | "reporter" | "developer" | "maintainer" | "owner" | "unknown";

export interface RepositoryPermissionDecision {
  actions: Record<string, boolean>;
  capabilities: Record<string, boolean>;
}

export interface RepositoryPermissions {
  role: RepositoryRole;
  roleLabel: string;
  isSuperAdmin: boolean;
  canRead: boolean;
  canReport: boolean;
  canWrite: boolean;
  canMerge: boolean;
  canOwn: boolean;
  repositoryPush: boolean;
  repositoryAdmin: boolean;
  issueCreate: boolean;
  issueWrite: boolean;
  issueComment: boolean;
  mergeRequestCreate: boolean;
  mergeRequestWrite: boolean;
  mergeRequestComment: boolean;
  mergeRequestMerge: boolean;
  packageWrite: boolean;
  wikiWrite: boolean;
  ciRead: boolean;
  ciWrite: boolean;
  jobRead: boolean;
  jobWrite: boolean;
  runnerRead: boolean;
  runnerAdmin: boolean;
  auditRead: boolean;
  projectDelete: boolean;
}

const reportRoles = new Set<RepositoryRole>(["guest", "reporter", "developer", "maintainer", "owner"]);
const writeRoles = new Set<RepositoryRole>(["developer", "maintainer", "owner"]);
const mergeRoles = new Set<RepositoryRole>(["maintainer", "owner"]);
const ownerRoles = new Set<RepositoryRole>(["owner"]);

export const buildRepositoryPermissions = (
  role: string | null | undefined,
  isSuperAdmin: boolean,
  permissionPayload?: unknown,
): RepositoryPermissions => {
  const normalizedRole = normalizeRepositoryRole(role);
  const canReport = isSuperAdmin || reportRoles.has(normalizedRole);
  const canWrite = isSuperAdmin || writeRoles.has(normalizedRole);
  const canMerge = isSuperAdmin || mergeRoles.has(normalizedRole);
  const canOwn = isSuperAdmin || ownerRoles.has(normalizedRole);
  const fallback: RepositoryPermissions = {
    role: normalizedRole,
    roleLabel: isSuperAdmin ? "super-admin" : normalizedRole,
    isSuperAdmin,
    canRead: true,
    canReport,
    canWrite,
    canMerge,
    canOwn,
    repositoryPush: canWrite,
    repositoryAdmin: canMerge,
    issueCreate: canReport,
    issueWrite: canWrite,
    issueComment: canReport,
    mergeRequestCreate: canWrite,
    mergeRequestWrite: canWrite,
    mergeRequestComment: canReport,
    mergeRequestMerge: canMerge,
    packageWrite: canWrite,
    wikiWrite: canWrite,
    ciRead: canReport,
    ciWrite: canWrite,
    jobRead: canReport,
    jobWrite: canWrite,
    runnerRead: canReport,
    runnerAdmin: canMerge,
    auditRead: canWrite,
    projectDelete: canOwn,
  };
  const decision = normalizeRepositoryPermissionDecision(permissionPayload);
  if (!decision) {
    return fallback;
  }

  return {
    ...fallback,
    canRead: actionAllowed(decision, "project.read", fallback.canRead),
    canReport: actionAllowed(decision, "project.jobs.read", fallback.canReport),
    canWrite: actionAllowed(decision, "project.write", fallback.canWrite),
    canMerge: actionAllowed(decision, "project.merge_requests.merge", fallback.canMerge),
    canOwn: actionAllowed(decision, "project.delete", fallback.canOwn),
    repositoryPush: actionAllowed(decision, "project.repository.push", fallback.repositoryPush),
    repositoryAdmin: actionAllowed(decision, "project.repository.admin", fallback.repositoryAdmin),
    issueCreate: actionAllowed(decision, "project.issues.create", fallback.issueCreate),
    issueWrite: actionAllowed(decision, "project.issues.write", fallback.issueWrite),
    issueComment: actionAllowed(decision, "project.issues.comment", fallback.issueComment),
    mergeRequestCreate: actionAllowed(decision, "project.merge_requests.create", fallback.mergeRequestCreate),
    mergeRequestWrite: actionAllowed(decision, "project.merge_requests.write", fallback.mergeRequestWrite),
    mergeRequestComment: actionAllowed(decision, "project.merge_requests.comment", fallback.mergeRequestComment),
    mergeRequestMerge: actionAllowed(decision, "project.merge_requests.merge", fallback.mergeRequestMerge),
    packageWrite: actionAllowed(decision, "project.packages.write", fallback.packageWrite),
    wikiWrite: actionAllowed(decision, "project.wiki.write", fallback.wikiWrite),
    ciRead: capabilityAllowed(decision, "ci_read", fallback.ciRead),
    ciWrite: capabilityAllowed(decision, "ci_write", fallback.ciWrite),
    jobRead: actionAllowed(decision, "project.jobs.read", fallback.jobRead),
    jobWrite: actionAllowed(decision, "project.jobs.write", fallback.jobWrite),
    runnerRead: actionAllowed(decision, "project.runners.read", fallback.runnerRead),
    runnerAdmin: actionAllowed(decision, "project.runners.admin", fallback.runnerAdmin),
    auditRead: capabilityAllowed(decision, "audit_read", fallback.auditRead),
    projectDelete: actionAllowed(decision, "project.delete", fallback.projectDelete),
  };
};

export const normalizeRepositoryRole = (role: string | null | undefined): RepositoryRole => {
  const normalized = role?.trim().toLowerCase();
  if (!normalized) {
    return "unknown";
  }
  if (normalized === "member") {
    return "developer";
  }
  if (
    normalized === "guest"
    || normalized === "reporter"
    || normalized === "developer"
    || normalized === "maintainer"
    || normalized === "owner"
  ) {
    return normalized;
  }
  return "unknown";
};

export const normalizeRepositoryPermissionDecision = (payload: unknown): RepositoryPermissionDecision | null => {
  const raw = resolveBody(payload);
  if (!isRecord(raw)) {
    return null;
  }
  return {
    actions: normalizeBooleanMap(raw.actions ?? raw.Actions),
    capabilities: normalizeBooleanMap(raw.capabilities ?? raw.Capabilities),
  };
};

const normalizeBooleanMap = (value: unknown): Record<string, boolean> => {
  if (!isRecord(value)) {
    return {};
  }
  return Object.fromEntries(Object.entries(value).map(([key, allowed]) => [key, normalizeBoolean(allowed)]));
};

const actionAllowed = (decision: RepositoryPermissionDecision, action: string, fallback: boolean): boolean =>
  Object.prototype.hasOwnProperty.call(decision.actions, action) ? decision.actions[action] : fallback;

const capabilityAllowed = (decision: RepositoryPermissionDecision, capability: string, fallback: boolean): boolean =>
  Object.prototype.hasOwnProperty.call(decision.capabilities, capability) ? decision.capabilities[capability] : fallback;
