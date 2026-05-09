import { ArrowRightLeft, FolderGit2, House, Shield, User } from "lucide-react";
import { NavLink, useLocation } from "react-router-dom";
import { useI18n } from "@/lib/i18n";
import { NavUser } from "@/components/nav-user";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";

interface AppSidebarProps extends React.ComponentProps<typeof Sidebar> {
  user: {
    name: string;
    email: string;
    isSuperAdmin: boolean;
  };
  onLogout: () => void;
}

export function AppSidebar({ user, onLogout, ...props }: AppSidebarProps) {
  const { t } = useI18n();
  const location = useLocation();
  const mainNavItems = [
    { title: t("Dashboard"), url: "/app/dashboard", icon: House },
    { title: t("My Repositories"), url: "/app/repositories", icon: FolderGit2 },
    { title: t("Profile"), url: "/app/profile", icon: User },
  ] as const;

  return (
    <Sidebar variant="inset" className="text-sidebar-foreground" {...props}>
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton size="lg" asChild>
              <NavLink to="/app/dashboard">
                <div className="relative flex aspect-square size-9 items-center justify-center rounded-xl bg-sidebar-primary text-sidebar-primary-foreground shadow-[0_16px_36px_-22px_hsl(var(--sidebar-primary))]">
                  <ArrowRightLeft className="size-4" />
                  <span className="absolute -right-0.5 -top-0.5 size-2 rounded-full bg-amber-400 ring-2 ring-sidebar" />
                </div>
                <div className="grid flex-1 text-left text-sm leading-tight">
                  <span className="truncate text-base font-semibold tracking-[-0.03em]">Gity</span>
                  <span className="truncate text-xs text-sidebar-foreground/65">{t("User Portal")}</span>
                </div>
              </NavLink>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
        <div className="mx-2 rounded-2xl border border-sidebar-border/70 bg-sidebar-accent/55 p-3 text-xs text-sidebar-foreground/75 shadow-inner group-data-[collapsible=icon]:hidden">
          <div className="mb-2 flex items-center justify-between">
            <span className="font-semibold text-sidebar-foreground">{t("Workspace")}</span>
            <span className="gity-dot bg-sidebar-primary" />
          </div>
          <p className="leading-5">{t("Unified workspace for auth, organization management, and repository operations.")}</p>
        </div>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>{t("Workspace")}</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {mainNavItems.map((item) => {
                const isActive = location.pathname.startsWith(item.url);
                return (
                  <SidebarMenuItem key={item.url}>
                    <SidebarMenuButton asChild isActive={isActive} tooltip={item.title}>
                      <NavLink to={item.url}>
                        <item.icon />
                        <span>{item.title}</span>
                      </NavLink>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                );
              })}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        {user.isSuperAdmin ? (
          <SidebarGroup>
            <SidebarGroupLabel>{t("Admin Side")}</SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                <SidebarMenuItem>
                  <SidebarMenuButton asChild isActive={location.pathname.startsWith("/admin")} tooltip={t("Admin Dashboard")}>
                    <NavLink to="/admin">
                      <Shield />
                      <span>{t("Admin Dashboard")}</span>
                    </NavLink>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
        ) : null}
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={user} onLogout={onLogout} />
      </SidebarFooter>
    </Sidebar>
  );
}
