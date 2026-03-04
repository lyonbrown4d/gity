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
  const currentTitle = t(routeTitleMap[location.pathname] ?? "Workspace");

  return (
    <SidebarProvider>
      <AppSidebar
        user={{
          name: identity?.name ?? t("Unknown User"),
          email: identity?.email ?? t("Unknown Email"),
          isSuperAdmin: Boolean(permissions?.isSuperAdmin),
        }}
        onLogout={() => logout()}
      />
      <SidebarInset>
        <header className="flex h-16 shrink-0 items-center justify-between gap-2 border-b px-4">
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
                <BreadcrumbPage>{currentTitle}</BreadcrumbPage>
              </BreadcrumbItem>
            </BreadcrumbList>
          </Breadcrumb>
          </div>
          <ViewControls compact />
        </header>
        <div className="flex flex-1 flex-col gap-4 p-4 pt-4 md:p-6 page-enter">
          <Outlet />
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}
