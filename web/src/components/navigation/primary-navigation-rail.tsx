import { ArrowRightLeft, FolderGit2, House, Inbox, Shield } from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { NavLink } from "react-router";
import { NavUser } from "@/components/nav-user";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import type { NavigationArea } from "./navigation-model";

interface PrimaryNavigationRailProps {
  activeArea: NavigationArea;
  user: { name: string; email: string; isSuperAdmin: boolean };
  onLogout: () => void;
  onNavigate: () => void;
  t: (value: string) => string;
}

interface RailItem {
  area: NavigationArea;
  title: string;
  label?: string;
  url: string;
  icon: LucideIcon;
}

export function PrimaryNavigationRail({
  activeArea,
  user,
  onLogout,
  onNavigate,
  t,
}: PrimaryNavigationRailProps): JSX.Element {
  const items: RailItem[] = [
    { area: "workspace", title: t("Workspace"), label: t("Home"), url: "/app/dashboard", icon: House },
    { area: "projects", title: t("Projects"), url: "/app/projects", icon: FolderGit2 },
    { area: "inbox", title: t("Inbox"), url: "/app/todos", icon: Inbox },
  ];

  return (
    <div className="flex w-16 shrink-0 flex-col items-center border-r border-sidebar-border bg-sidebar text-sidebar-foreground">
      <NavLink
        to="/app/dashboard"
        aria-label="Gity"
        className="my-2.5 flex size-10 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground shadow-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sidebar-ring"
        onClick={onNavigate}
      >
        <ArrowRightLeft className="size-4" aria-hidden="true" />
      </NavLink>

      <nav aria-label={t("Primary navigation")} className="flex flex-1 flex-col items-center gap-1.5 py-2">
        {items.map((item) => (
          <RailLink key={item.area} item={item} active={activeArea === item.area} onNavigate={onNavigate} />
        ))}
      </nav>

      <div className="mb-2 flex flex-col items-center gap-1 border-t border-sidebar-border pt-2">
        {user.isSuperAdmin ? (
          <RailLink
            item={{ area: "admin", title: t("Admin"), url: "/admin", icon: Shield }}
            active={false}
            onNavigate={onNavigate}
          />
        ) : null}
        <NavUser compact user={user} onLogout={onLogout} onNavigate={onNavigate} />
      </div>
    </div>
  );
}

function RailLink({ item, active, onNavigate }: {
  item: RailItem;
  active: boolean;
  onNavigate: () => void;
}): JSX.Element {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <NavLink
          to={item.url}
          aria-label={item.title}
          aria-current={active ? "page" : undefined}
          onClick={onNavigate}
          className={cn(
            "flex w-12 flex-col items-center gap-1 rounded-lg py-2 text-sidebar-foreground/60 outline-none transition-colors hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 focus-visible:ring-sidebar-ring",
            active && "bg-sidebar-accent text-sidebar-primary shadow-sm",
          )}
        >
          <item.icon className="size-4" aria-hidden="true" />
          <span className="max-w-11 truncate text-[9px] font-medium leading-none">{item.label ?? item.title}</span>
        </NavLink>
      </TooltipTrigger>
      <TooltipContent side="right">{item.title}</TooltipContent>
    </Tooltip>
  );
}
