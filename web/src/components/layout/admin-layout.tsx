import { Boxes, Building2, LayoutDashboard, LogOut, ShieldCheck, UserCircle2, Users } from "lucide-react";
import { motion, useReducedMotion } from "motion/react";
import { NavLink, Outlet, useLocation, useNavigate } from "react-router";
import { useGetIdentity, useLogout, usePermissions } from "@refinedev/core";
import { ViewControls } from "@/components/common/view-controls";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { useI18n } from "@/lib/i18n";
import { cn } from "@/lib/utils";

const routeTitles: Record<string, string> = {
  "/admin": "Overview",
  "/admin/orgs": "Organizations",
  "/admin/projects": "Projects",
  "/admin/users": "Users",
};

export function AdminLayout(): JSX.Element {
  const { t } = useI18n();
  const location = useLocation();
  const navigate = useNavigate();
  const { mutate: logout } = useLogout();
  const { data: identity } = useGetIdentity<{ name?: string; isSuperAdmin?: boolean }>({});
  const { data: permissions } = usePermissions<{ isSuperAdmin?: boolean }>({});
  const reduceMotion = useReducedMotion();
  const baseMenus = [
    { to: "/admin", label: t("Overview"), icon: LayoutDashboard },
    { to: "/admin/orgs", label: t("Organizations"), icon: Building2 },
    { to: "/admin/projects", label: t("Projects"), icon: Boxes },
  ];
  const menuItems = permissions?.isSuperAdmin
    ? [...baseMenus, { to: "/admin/users", label: t("Users"), icon: Users }]
    : baseMenus;
  const currentTitle = t(routeTitles[location.pathname] ?? "Overview");
  const signOut = () => logout(undefined, {
    onSuccess: ({ success }) => {
      if (success) navigate("/login", { replace: true });
    },
  });

  return (
    <div className="min-h-screen bg-background text-foreground">
      <a href="#admin-main-content" className="skip-link">
        {t("Skip to main content")}
      </a>
      <div className="mx-auto flex min-h-screen w-full max-w-[1600px] border-x border-border/60 bg-background/75">
        <aside className="hidden w-56 shrink-0 flex-col border-r border-sidebar-border bg-sidebar text-sidebar-foreground md:flex">
          <div className="flex h-14 items-center gap-3 border-b border-sidebar-border px-3">
            <span className="grid size-8 place-items-center rounded-md border border-sidebar-border bg-sidebar-accent font-mono text-xs font-bold text-sidebar-primary">
              G_
            </span>
            <div className="min-w-0">
              <p className="truncate text-sm font-semibold">{t("Gity Console")}</p>
              <p className="font-mono text-[9px] uppercase tracking-[0.18em] text-sidebar-foreground/55">{t("Admin Side")}</p>
            </div>
          </div>
          <nav className="flex-1 p-2" aria-label={t("Admin Side")}>
            <p className="px-2 pb-2 pt-1 font-mono text-[9px] uppercase tracking-[0.18em] text-sidebar-foreground/45">{t("Platform")}</p>
            <div className="space-y-0.5">
              {menuItems.map((item) => <AdminNavLink key={item.to} {...item} />)}
            </div>
          </nav>
          <div className="border-t border-sidebar-border p-2">
            <div className="mb-2 flex items-center gap-2 rounded-md bg-sidebar-accent/70 px-2 py-2">
              <UserCircle2 className="size-4 shrink-0 text-sidebar-foreground/60" />
              <div className="min-w-0 flex-1">
                <p className="truncate text-xs font-medium">{identity?.name ?? t("User")}</p>
                <p className="truncate font-mono text-[9px] text-sidebar-foreground/50">
                  {identity?.isSuperAdmin ? t("Super Admin") : t("Admin Side")}
                </p>
              </div>
              <span className="size-1.5 rounded-full bg-sidebar-primary" aria-hidden="true" />
            </div>
            <Button variant="ghost" size="sm" onClick={signOut} className="h-8 w-full justify-start text-xs text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground">
              <LogOut className="size-3.5" />
              {t("Logout")}
            </Button>
          </div>
        </aside>
        <div className="min-w-0 flex-1">
          <header className="sticky top-0 z-20 border-b border-border/70 bg-background/90 backdrop-blur-xl">
            <div className="flex h-14 items-center justify-between gap-3 px-3 sm:px-5">
              <div className="flex min-w-0 items-center gap-2">
                <ShieldCheck className="size-4 text-primary" aria-hidden="true" />
                <p className="truncate font-mono text-xs text-muted-foreground">
                  <span className="hidden sm:inline">admin / </span>
                  <span className="font-semibold text-foreground">{currentTitle}</span>
                </p>
                {identity?.isSuperAdmin ? <Badge variant="secondary" className="hidden text-[9px] sm:inline-flex">{t("Super Admin")}</Badge> : null}
              </div>
              <div className="flex items-center gap-1.5">
                <ViewControls compact />
                <Button variant="outline" size="icon" onClick={signOut} className="size-8 md:hidden" aria-label={t("Logout")}>
                  <LogOut className="size-3.5" />
                </Button>
              </div>
            </div>
            <nav className="flex gap-1 overflow-x-auto border-t border-border/60 px-2 py-1 md:hidden" aria-label={t("Admin Side")}>
              {menuItems.map((item) => <AdminNavLink key={item.to} {...item} mobile />)}
            </nav>
          </header>
          <motion.main
            key={location.pathname}
            id="admin-main-content"
            className="p-3 sm:p-5 lg:p-6"
            initial={reduceMotion ? false : { opacity: 0, y: 6 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.18, ease: [0.16, 1, 0.3, 1] }}
          >
            <Outlet />
          </motion.main>
        </div>
      </div>
    </div>
  );
}

interface AdminNavLinkProps {
  to: string;
  label: string;
  icon: typeof LayoutDashboard;
  mobile?: boolean;
}

function AdminNavLink({ to, label, icon: Icon, mobile = false }: AdminNavLinkProps): JSX.Element {
  return (
    <NavLink
      to={to}
      end={to === "/admin"}
      className={({ isActive }) => cn(
        "group flex items-center gap-2 rounded-md text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sidebar-ring",
        mobile ? "h-8 shrink-0 px-2.5" : "h-9 px-2",
        isActive
          ? mobile ? "bg-primary text-primary-foreground" : "bg-sidebar-accent text-sidebar-accent-foreground"
          : mobile ? "text-muted-foreground hover:bg-muted hover:text-foreground" : "text-sidebar-foreground/65 hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
      )}
    >
      <Icon className="size-3.5" />
      <span>{label}</span>
      {!mobile ? <span className="ml-auto size-1 rounded-full bg-sidebar-primary opacity-0 group-aria-[current=page]:opacity-100" /> : null}
    </NavLink>
  );
}
