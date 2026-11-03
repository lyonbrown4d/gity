import { ChevronDown, type LucideIcon } from "lucide-react";
import { NavLink, useLocation } from "react-router";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import {
  SidebarGroup, SidebarGroupContent, SidebarGroupLabel, SidebarMenu,
  SidebarMenuButton, SidebarMenuItem,
} from "@/components/ui/sidebar";

export interface ContextNavigationItem {
  title: string;
  url: string;
  icon: LucideIcon;
}

interface ContextNavigationGroupProps {
  label: string;
  items: ContextNavigationItem[];
  activeUrl?: string;
  collapsible?: boolean;
  onNavigate: () => void;
}

export const ContextNavigationGroup = ({
  label,
  items,
  activeUrl,
  collapsible = false,
  onNavigate,
}: ContextNavigationGroupProps): JSX.Element => {
  const content = (
    <SidebarGroupContent>
      <SidebarMenu className="gap-0.5">
        {items.map((item) => (
          <ContextLink key={item.url} item={item} activeUrl={activeUrl} onNavigate={onNavigate} />
        ))}
      </SidebarMenu>
    </SidebarGroupContent>
  );

  if (!collapsible) {
    return (
        <SidebarGroup className="px-2 py-1.5">
        <SidebarGroupLabel className="h-7 px-2 font-mono text-[9px] uppercase tracking-[0.16em] text-sidebar-foreground/50">
          {label}
        </SidebarGroupLabel>
        {content}
      </SidebarGroup>
    );
  }

  return (
    <Collapsible defaultOpen className="group/collapsible">
      <SidebarGroup className="px-2 py-1">
        <SidebarGroupLabel asChild className="h-7 px-2 font-mono text-[9px] uppercase tracking-[0.16em] text-sidebar-foreground/50">
          <CollapsibleTrigger>
            {label}
            <ChevronDown className="ml-auto size-3 transition-transform duration-200 group-data-[state=open]/collapsible:rotate-180" />
          </CollapsibleTrigger>
        </SidebarGroupLabel>
        <CollapsibleContent>{content}</CollapsibleContent>
      </SidebarGroup>
    </Collapsible>
  );
};

const ContextLink = ({ item, activeUrl, onNavigate }: {
  item: ContextNavigationItem;
  activeUrl?: string;
  onNavigate: () => void;
}): JSX.Element => {
  const location = useLocation();
  const itemPath = item.url.split("?")[0];
  const currentUrl = `${location.pathname}${location.search}`;
  const active = activeUrl ? activeUrl === item.url : currentUrl === item.url || location.pathname === itemPath;

  return (
    <SidebarMenuItem>
      <SidebarMenuButton
        asChild
        isActive={active}
        tooltip={item.title}
        className="h-9 rounded-md px-2 text-sidebar-foreground/70 hover:bg-sidebar-accent hover:text-sidebar-accent-foreground data-[active=true]:bg-sidebar-accent data-[active=true]:font-medium data-[active=true]:text-sidebar-accent-foreground"
      >
        <NavLink to={item.url} aria-label={item.title} onClick={onNavigate}>
          <span className={active ? "flex size-6 items-center justify-center rounded-md bg-sidebar-primary/15 text-sidebar-primary group-data-[collapsible=icon]:size-4" : "flex size-6 items-center justify-center rounded-md group-data-[collapsible=icon]:size-4"}>
            <item.icon className="size-3.5 group-data-[collapsible=icon]:size-3" aria-hidden="true" />
          </span>
          <span className="truncate group-data-[collapsible=icon]:hidden">{item.title}</span>
        </NavLink>
      </SidebarMenuButton>
    </SidebarMenuItem>
  );
};
