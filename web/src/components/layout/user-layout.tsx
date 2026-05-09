import { Outlet, useLocation } from "react-router-dom";
import { useGetIdentity, useLogout, usePermissions } from "@refinedev/core";
import { AppSidebar } from "@/components/app-sidebar";
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
  "/app/repositories": "My Repositories",
  "/app/profile": "Profile",
};

export function UserLayout(): JSX.Element {
  const { t } = useI18n();
  const location = useLocation();
  const { mutate: logout } = useLogout();
  const { data: identity } = useGetIdentity<{ name?: string; email?: string; isSuperAdmin?: boolean }>();
  const { data: permissions } = usePermissions<{ isSuperAdmin?: boolean }>();
  const currentTitle = t(
    location.pathname.startsWith("/app/repositories/")
      ? "Repository"
      : (routeTitleMap[location.pathname] ?? "Workspace"),
  );

  return (
    <SidebarProvider className="gity-shell">
      <AppSidebar
        user={{
          name: identity?.name ?? t("Unknown User"),
          email: identity?.email ?? t("Unknown Email"),
          isSuperAdmin: Boolean(permissions?.isSuperAdmin),
        }}
        onLogout={() => logout()}
      />
      <SidebarInset className="overflow-hidden bg-background/55 backdrop-blur-xl md:border md:border-border/60 md:bg-background/60">
        <header className="sticky top-0 z-20 flex h-16 shrink-0 items-center justify-between gap-2 border-b border-border/70 bg-background/75 px-4 backdrop-blur-xl">
          <div className="flex items-center gap-2">
            <SidebarTrigger className="-ml-1" />
            <Separator orientation="vertical" className="mr-2 h-4" />
            <Breadcrumb>
              <BreadcrumbList>
                <BreadcrumbItem className="hidden md:block">
                  {t("User Workspace")}
                </BreadcrumbItem>
                <BreadcrumbSeparator className="hidden md:block" />
                <BreadcrumbItem>
                  <BreadcrumbPage className="font-semibold">{currentTitle}</BreadcrumbPage>
                </BreadcrumbItem>
              </BreadcrumbList>
            </Breadcrumb>
          </div>
          <ViewControls compact />
        </header>
        <div className="flex flex-1 flex-col gap-5 p-4 pt-4 md:p-6 page-enter">
          <Outlet />
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}
