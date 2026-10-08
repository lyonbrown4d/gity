import type { CSSProperties } from "react";
import { Outlet, useLocation, useNavigate } from "react-router";
import { useGetIdentity, useLogout, usePermissions } from "@refinedev/core";
import { AppSidebar } from "@/components/app-sidebar";
import { GlobalQuickJump } from "@/components/global-quick-jump";
import { ViewControls } from "@/components/common/view-controls";
import { useI18n } from "@/lib/i18n";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { Separator } from "@/components/ui/separator";
import { SidebarInset, SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";

const routeTitleMap: Record<string, string> = {
  "/app/dashboard": "Dashboard",
  "/app/todos": "Inbox",
  "/app/projects": "My Projects",
  "/app/repositories": "My Projects",
  "/app/profile": "Profile",
};

export function UserLayout(): JSX.Element {
  const { t } = useI18n();
  const location = useLocation();
  const navigate = useNavigate();
  const { mutate: logout } = useLogout();
  const { data: identity } = useGetIdentity<{ name?: string; email?: string; isSuperAdmin?: boolean }>({});
  const { data: permissions } = usePermissions<{ isSuperAdmin?: boolean }>({});
  const currentTitle = t(
    location.pathname.startsWith("/app/projects/") || location.pathname.startsWith("/app/repositories/")
      ? "Project"
      : (routeTitleMap[location.pathname] ?? "Workspace"),
  );

  return (
    <SidebarProvider
      className="gity-shell bg-background"
      style={{ "--sidebar-width": "19rem", "--sidebar-width-icon": "3.5rem" } as CSSProperties}
    >
      <a href="#main-content" className="skip-link">
        {t("Skip to main content")}
      </a>
      <AppSidebar
        user={{
          name: identity?.name ?? t("Unknown User"),
          email: identity?.email ?? t("Unknown Email"),
          isSuperAdmin: Boolean(permissions?.isSuperAdmin),
        }}
        onLogout={() =>
          logout(undefined, {
            onSuccess: ({ success }) => {
              if (success) {
                navigate("/login", { replace: true });
              }
            },
          })
        }
      />
      <SidebarInset className="min-w-0 overflow-hidden bg-background md:rounded-none md:shadow-none">
        <header className="sticky top-0 z-20 grid h-12 shrink-0 grid-cols-[minmax(0,1fr)_auto_auto] items-center gap-2 border-b border-border bg-background px-3 md:grid-cols-[minmax(12rem,1fr)_minmax(18rem,32rem)_minmax(12rem,1fr)]">
          <div className="flex min-w-0 items-center gap-2">
            <SidebarTrigger className="-ml-1 shrink-0" aria-label={t("Toggle Sidebar")} />
            <Separator orientation="vertical" className="h-4" />
            <Breadcrumb className="min-w-0">
              <BreadcrumbList className="flex-nowrap">
                <BreadcrumbItem className="hidden lg:block">{t("User Workspace")}</BreadcrumbItem>
                <BreadcrumbSeparator className="hidden lg:block" />
                <BreadcrumbItem className="min-w-0">
                  <BreadcrumbPage className="truncate text-sm font-medium">{currentTitle}</BreadcrumbPage>
                </BreadcrumbItem>
              </BreadcrumbList>
            </Breadcrumb>
          </div>
          <div className="flex min-w-0 justify-end md:justify-center">
            <GlobalQuickJump canAccessAdmin={Boolean(permissions?.isSuperAdmin)} />
          </div>
          <div className="flex items-center justify-end gap-1">
            <ViewControls compact />
          </div>
        </header>
        <main id="main-content" className="flex flex-1 flex-col gap-4 p-3 md:p-5 page-enter">
          <Outlet />
        </main>
      </SidebarInset>
    </SidebarProvider>
  );
}
