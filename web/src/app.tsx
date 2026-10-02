import { Suspense, lazy, type ReactNode } from "react";
import { Refine, Authenticated, usePermissions } from "@refinedev/core";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Navigate, Route, Routes } from "react-router";
import { useI18n } from "@/lib/i18n";
import { authProvider } from "@/providers/auth-provider";
import { dataProvider } from "@/providers/data-provider";

const AdminLayout = lazy(() =>
  import("@/components/layout/admin-layout").then((module) => ({ default: module.AdminLayout })),
);
const UserLayout = lazy(() =>
  import("@/components/layout/user-layout").then((module) => ({ default: module.UserLayout })),
);
const LoginPage = lazy(() =>
  import("@/pages/login").then((module) => ({ default: module.LoginPage })),
);
const AdminOverviewPage = lazy(() =>
  import("@/pages/admin/overview-page").then((module) => ({ default: module.AdminOverviewPage })),
);
const AdminOrganizationsPage = lazy(() =>
  import("@/pages/admin/organizations-page").then((module) => ({ default: module.AdminOrganizationsPage })),
);
const AdminRepositoriesPage = lazy(() =>
  import("@/pages/admin/repositories-page").then((module) => ({ default: module.AdminRepositoriesPage })),
);
const AdminUsersPage = lazy(() =>
  import("@/pages/admin/users-page").then((module) => ({ default: module.AdminUsersPage })),
);
const AppDashboardPage = lazy(() =>
  import("@/pages/app/dashboard-page").then((module) => ({ default: module.AppDashboardPage })),
);
const AppRepositoriesPage = lazy(() =>
  import("@/pages/app/repositories-page").then((module) => ({ default: module.AppRepositoriesPage })),
);
const AppTodosPage = lazy(() =>
  import("@/pages/app/todos-page").then((module) => ({ default: module.AppTodosPage })),
);
const RepositoryDetailPage = lazy(() =>
  import("@/pages/app/repository-detail-page").then((module) => ({ default: module.RepositoryDetailPage })),
);
const RepositoryIssueDetailPage = lazy(() =>
  import("@/pages/app/repository-issue-detail-page").then((module) => ({ default: module.RepositoryIssueDetailPage })),
);
const AppProfilePage = lazy(() =>
  import("@/pages/app/profile-page").then((module) => ({ default: module.AppProfilePage })),
);

const queryClient = new QueryClient();

export function App(): JSX.Element {
  const { t } = useI18n();

  return (
    <BrowserRouter>
      <QueryClientProvider client={queryClient}>
        <Refine
          dataProvider={dataProvider}
          authProvider={authProvider}
          resources={[
            { name: "admin-overview", list: "/admin", meta: { label: t("Overview") } },
            { name: "organizations", list: "/admin/orgs", meta: { label: t("Organizations") } },
            { name: "projects", list: "/admin/projects", meta: { label: t("Projects") } },
            { name: "users", list: "/admin/users", meta: { label: t("Users") } },
            { name: "dashboard", list: "/app/dashboard", meta: { label: t("Dashboard") } },
            { name: "todos", list: "/app/todos", meta: { label: t("Inbox") } },
            { name: "my-projects", list: "/app/projects", meta: { label: t("My Projects") } },
            { name: "profile", list: "/app/profile", meta: { label: t("Profile") } },
          ]}
        >
          <Suspense
            fallback={
              <div className="flex min-h-[60vh] items-center justify-center text-sm text-muted-foreground">
                {t("Loading...")}
              </div>
            }
          >
            <Routes>
              <Route path="/login" element={<LoginPage />} />

              <Route
                element={
                  <Authenticated key="admin-layout" fallback={<Navigate to="/login" replace />}>
                    <RequireSuperAdmin>
                      <AdminLayout />
                    </RequireSuperAdmin>
                  </Authenticated>
                }
              >
                <Route path="/admin" element={<AdminOverviewPage />} />
                <Route path="/admin/orgs" element={<AdminOrganizationsPage />} />
                <Route path="/admin/projects" element={<AdminRepositoriesPage />} />
                <Route path="/admin/repos" element={<Navigate to="/admin/projects" replace />} />
                <Route path="/admin/users" element={<AdminUsersPage />} />
              </Route>

              <Route
                element={
                  <Authenticated key="user-layout" fallback={<Navigate to="/login" replace />}>
                    <UserLayout />
                  </Authenticated>
                }
              >
                <Route path="/app/dashboard" element={<AppDashboardPage />} />
                <Route path="/app/todos" element={<AppTodosPage />} />
                <Route path="/app/projects" element={<AppRepositoriesPage />} />
                <Route path="/app/repositories" element={<Navigate to="/app/projects" replace />} />
                <Route path="/app/projects/:organizationId/:projectId" element={<RepositoryDetailPage />} />
                <Route path="/app/repositories/:organizationId/:repoId" element={<RepositoryDetailPage />} />
                <Route
                  path="/app/projects/:organizationId/:projectId/issues/:issueNumber"
                  element={<RepositoryIssueDetailPage />}
                />
                <Route
                  path="/app/repositories/:organizationId/:repoId/issues/:issueNumber"
                  element={<RepositoryIssueDetailPage />}
                />
                <Route path="/app/profile" element={<AppProfilePage />} />
              </Route>

              <Route
                path="/"
                element={
                  <Authenticated key="root-redirect" fallback={<Navigate to="/login" replace />}>
                    <Navigate to="/app/dashboard" replace />
                  </Authenticated>
                }
              />
              <Route path="*" element={<Navigate to="/app/dashboard" replace />} />
            </Routes>
          </Suspense>
        </Refine>
      </QueryClientProvider>
    </BrowserRouter>
  );
}

function RequireSuperAdmin({ children }: { children: ReactNode }): JSX.Element {
  const { t } = useI18n();
  const { data: permissions, isLoading } = usePermissions<{ isSuperAdmin?: boolean }>({});

  if (isLoading) {
    return (
      <div className="flex min-h-[60vh] items-center justify-center text-sm text-muted-foreground">
        {t("Loading...")}
      </div>
    );
  }
  if (!permissions?.isSuperAdmin) {
    return <Navigate to="/app/dashboard" replace />;
  }
  return <>{children}</>;
}
