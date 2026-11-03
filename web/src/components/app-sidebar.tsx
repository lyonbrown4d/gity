import { useList } from "@refinedev/core";
import { useLocation } from "react-router";
import { ContextNavigationPanel } from "@/components/navigation/context-navigation-panel";
import { parseProjectRoute, resolveNavigationArea } from "@/components/navigation/navigation-model";
import { Sidebar, useSidebar } from "@/components/ui/sidebar";
import { useI18n } from "@/lib/i18n";
import type { RepositoryView } from "@/pages/types";

interface AppSidebarProps extends React.ComponentProps<typeof Sidebar> {
  user: { name: string; email: string; isSuperAdmin: boolean };
  onLogout: () => void;
}

export const AppSidebar = ({ user, onLogout, ...props }: AppSidebarProps): JSX.Element => {
  const { t } = useI18n();
  const location = useLocation();
  const { isMobile, setOpenMobile } = useSidebar();
  const activeArea = resolveNavigationArea(location.pathname);
  const projectRoute = parseProjectRoute(location.pathname);
  const projectSegments = location.pathname.split("/").filter(Boolean).slice(4);
  const routeTab = projectSegments[0] ?? new URLSearchParams(location.search).get("tab") ?? "overview";
  const currentTab = routeTab === "pipelines" && projectSegments[1] === "editor"
    ? "pipelines/editor"
    : routeTab;
  const todoState = new URLSearchParams(location.search).get("state") === "done" ? "done" : "pending";
  const projectQuery = useList<RepositoryView>({
    resource: "my-projects",
    pagination: { mode: "off" },
    queryOptions: { enabled: activeArea === "projects" },
  });
  const closeMobile = () => { if (isMobile) setOpenMobile(false); };

  return (
    <Sidebar collapsible="icon" variant="sidebar" className="gity-app-sidebar border-r border-sidebar-border" {...props}>
      <ContextNavigationPanel
        activeArea={activeArea}
        currentTab={currentTab}
        projectRoute={projectRoute}
        projects={projectQuery.result.data ?? []}
        todoState={todoState}
        user={user}
        onLogout={onLogout}
        onNavigate={closeMobile}
        t={t}
      />
    </Sidebar>
  );
};
