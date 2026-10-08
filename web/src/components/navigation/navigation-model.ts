export type NavigationArea = "workspace" | "projects" | "inbox" | "admin";

export interface ProjectRoute {
  organizationId: string;
  projectId: string;
}

export function resolveNavigationArea(pathname: string): NavigationArea {
  if (pathname.startsWith("/admin")) return "admin";
  if (pathname.startsWith("/app/projects") || pathname.startsWith("/app/repositories")) return "projects";
  if (pathname.startsWith("/app/todos")) return "inbox";
  return "workspace";
}

export function parseProjectRoute(pathname: string): ProjectRoute | null {
  const match = /^\/app\/(?:projects|repositories)\/([^/]+)\/([^/]+)/.exec(pathname);
  if (!match) return null;
  try {
    return { organizationId: decodeURIComponent(match[1]), projectId: decodeURIComponent(match[2]) };
  } catch {
    return { organizationId: match[1], projectId: match[2] };
  }
}

export function projectBasePath(route: ProjectRoute): string {
  return `/app/projects/${encodeURIComponent(route.organizationId)}/${encodeURIComponent(route.projectId)}`;
}
