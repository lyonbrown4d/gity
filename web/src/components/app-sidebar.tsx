import { useList } from "@refinedev/core";
import { useLocation } from "react-router";
import { ContextNavigationPanel } from "@/components/navigation/context-navigation-panel";
import { parseProjectRoute, resolveNavigationArea } from "@/components/navigation/navigation-model";
import { PrimaryNavigationRail } from "@/components/navigation/primary-navigation-rail";
import { Sidebar, useSidebar } from "@/components/ui/sidebar";
import { useI18n } from "@/lib/i18n";
import type { RepositoryView } from "@/pages/types";

interface AppSidebarProps extends React.ComponentProps<typeof Sidebar> {
  user: { name: string; email: string; isSuperAdmin: boolean };
  onLogout: () => void;
}

export function AppSidebar({ user, onLogout, ...props }: AppSidebarProps): JSX.Element {
  const { t } = useI18n();
  const location = useLocation();
  const { isMobile, setOpenMobile } = useSidebar();
  const activeArea = resolveNavigationArea(location.pathname);
  const projectRoute = parseProjectRoute(location.pathname);
  const currentTab = location.pathname.includes("/issues/")
    ? "issues"
    : new URLSearchParams(location.search).get("tab") ?? "overview";
  const todoState = new URLSearchParams(location.search).get("state") === "done" ? "done" : "pending";
  const projectQuery = useList<RepositoryView>({
    resource: "my-projects",
    queryOptions: { enabled: activeArea === "projects" },
  });
  const projects = projectQuery.result.data ?? [];
  const closeMobile = () => {
    if (isMobile) setOpenMobile(false);
  };

  return (
    <Sidebar collapsible="icon" variant="sidebar" className="gity-dual-sidebar border-r border-sidebar-border" {...props}>
      <div className="flex h-full min-h-0 w-full">
        <PrimaryNavigationRail
          activeArea={activeArea}
          user={user}
          onLogout={onLogout}
          onNavigate={closeMobile}
          t={t}
        />
        <ContextNavigationPanel
          activeArea={activeArea}
          currentTab={currentTab}
          projectRoute={projectRoute}
          projects={projects}
          todoState={todoState}
          user={user}
          onNavigate={closeMobile}
          t={t}
        />
      </div>
    </Sidebar>
  );
}
