export type RepositoryRole = "guest" | "reporter" | "developer" | "maintainer" | "owner" | "unknown";

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

export const buildRepositoryPermissions = (role: string | null | undefined, isSuperAdmin: boolean): RepositoryPermissions => {
  const normalizedRole = normalizeRepositoryRole(role);
  const canReport = isSuperAdmin || reportRoles.has(normalizedRole);
  const canWrite = isSuperAdmin || writeRoles.has(normalizedRole);
  const canMerge = isSuperAdmin || mergeRoles.has(normalizedRole);
  const canOwn = isSuperAdmin || ownerRoles.has(normalizedRole);

  return {
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
