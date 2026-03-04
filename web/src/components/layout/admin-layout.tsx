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
  const { data: identity } = useGetIdentity<{ name?: string; isSuperAdmin?: boolean }>();
  const { data: permissions } = usePermissions<{ isSuperAdmin?: boolean }>();
  const adminMenus = [
    { to: "/admin", label: t("Overview"), icon: Shield },
    { to: "/admin/orgs", label: t("Organizations"), icon: Building2 },
    { to: "/admin/repos", label: t("Repositories"), icon: Boxes },
  ];
  const menuItems = permissions?.isSuperAdmin
    ? [...adminMenus, { to: "/admin/users", label: t("Users"), icon: Users }]
    : adminMenus;

  return (
    <div className="min-h-screen bg-[radial-gradient(circle_at_10%_0%,hsl(35_58%_90%),transparent_40%),radial-gradient(circle_at_100%_100%,hsl(160_22%_90%),transparent_35%)]">
      <div className="mx-auto flex min-h-screen max-w-[1400px]">
        <aside className="w-72 border-r bg-background/90 backdrop-blur">
          <div className="p-6">
            <p className="text-xs uppercase tracking-[0.2em] text-muted-foreground">{t("Refine Admin")}</p>
            <h1 className="mt-2 text-2xl font-semibold">{t("Gity Console")}</h1>
          </div>
          <Separator />
          <nav className="space-y-1 p-4">
            {menuItems.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                end={item.to === "/admin"}
                className={({ isActive }) =>
                  cn(
                    "flex items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors",
                    isActive ? "bg-primary text-primary-foreground" : "hover:bg-accent",
                  )
                }
              >
                <item.icon className="h-4 w-4" />
                <span>{item.label}</span>
              </NavLink>
            ))}
          </nav>
        </aside>
        <main className="flex-1">
          <header className="flex items-center justify-between border-b bg-background/80 px-8 py-4 backdrop-blur">
            <div>
              <p className="text-xs uppercase tracking-[0.2em] text-muted-foreground">{t("Admin Side")}</p>
              <p className="text-sm text-muted-foreground">{t("Refine-driven routing and auth flow")}</p>
            </div>
            <div className="flex items-center gap-3">
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
          <div className="p-8 page-enter">
            <Outlet />
          </div>
        </main>
      </div>
    </div>
  );
}
