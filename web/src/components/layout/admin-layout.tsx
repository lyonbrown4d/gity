import { LogOut, Shield, Boxes, Building2, UserCircle2, Users } from "lucide-react";
import { NavLink, Outlet } from "react-router-dom";
import { useGetIdentity, useLogout, usePermissions } from "@refinedev/core";
import { ViewControls } from "@/components/common/view-controls";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { useI18n } from "@/lib/i18n";
import { cn } from "@/lib/utils";

export function AdminLayout(): JSX.Element {
  const { t } = useI18n();
  const { mutate: logout } = useLogout();
  const { data: identity } = useGetIdentity<{ name?: string; isSuperAdmin?: boolean }>({});
  const { data: permissions } = usePermissions<{ isSuperAdmin?: boolean }>({});
  const adminMenus = [
    { to: "/admin", label: t("Overview"), icon: Shield },
    { to: "/admin/orgs", label: t("Organizations"), icon: Building2 },
    { to: "/admin/projects", label: t("Projects"), icon: Boxes },
  ];
  const menuItems = permissions?.isSuperAdmin
    ? [...adminMenus, { to: "/admin/users", label: t("Users"), icon: Users }]
    : adminMenus;

  return (
    <div className="min-h-screen bg-background">
      <a href="#admin-main-content" className="skip-link">
        {t("Skip to main content")}
      </a>
      <div className="mx-auto flex min-h-screen max-w-[1400px]">
        <aside className="hidden w-72 border-r bg-sidebar text-sidebar-foreground lg:block">
          <div className="p-6">
            <p className="text-xs font-semibold uppercase text-sidebar-foreground/65">{t("Refine Admin")}</p>
            <h1 className="mt-2 text-2xl font-semibold">{t("Gity Console")}</h1>
          </div>
          <Separator />
          <nav className="flex flex-col gap-1 p-4">
            {menuItems.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                end={item.to === "/admin"}
                className={({ isActive }) =>
                  cn(
                    "flex items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors",
                    isActive ? "bg-sidebar-primary text-sidebar-primary-foreground" : "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
                  )
                }
              >
                <item.icon className="h-4 w-4" />
                <span>{item.label}</span>
              </NavLink>
            ))}
          </nav>
        </aside>
        <main id="admin-main-content" className="flex-1">
          <header className="flex flex-col gap-4 border-b bg-background/85 px-4 py-4 backdrop-blur md:px-8 lg:flex-row lg:items-center lg:justify-between">
            <div>
              <p className="text-xs font-semibold uppercase text-muted-foreground">{t("Admin Side")}</p>
              <p className="text-sm text-muted-foreground">{t("Refine-driven routing and auth flow")}</p>
            </div>
            <nav className="flex gap-2 overflow-x-auto lg:hidden">
              {menuItems.map((item) => (
                <Button key={item.to} variant="outline" size="sm" asChild>
                  <NavLink to={item.to} end={item.to === "/admin"}>
                    <item.icon className="h-4 w-4" />
                    {item.label}
                  </NavLink>
                </Button>
              ))}
            </nav>
            <div className="flex flex-wrap items-center gap-3">
              <ViewControls compact />
              <div className="flex items-center gap-2 rounded-md border px-3 py-2">
                <UserCircle2 className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm">{identity?.name ?? t("User")}</span>
                {identity?.isSuperAdmin ? <Badge variant="secondary">{t("Super Admin")}</Badge> : null}
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={() => logout()}
                className="gap-2 action-pop"
              >
                <LogOut className="h-4 w-4" />
                {t("Logout")}
              </Button>
            </div>
          </header>
          <div className="p-4 md:p-8 page-enter">
            <Outlet />
          </div>
        </main>
      </div>
    </div>
  );
}
