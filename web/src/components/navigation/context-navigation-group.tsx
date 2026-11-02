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

export function ContextNavigationGroup({
  label,
  items,
  activeUrl,
  collapsible = false,
  onNavigate,
}: ContextNavigationGroupProps): JSX.Element {
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
        <SidebarGroupLabel className="h-7 px-2 font-mono text-[9px] uppercase tracking-[0.16em] text-muted-foreground">
          {label}
        </SidebarGroupLabel>
        {content}
      </SidebarGroup>
    );
  }

  return (
    <Collapsible defaultOpen className="group/collapsible">
      <SidebarGroup className="px-2 py-1">
        <SidebarGroupLabel asChild className="h-7 px-2 font-mono text-[9px] uppercase tracking-[0.16em] text-muted-foreground">
          <CollapsibleTrigger>
            {label}
            <ChevronDown className="ml-auto size-3 transition-transform duration-200 group-data-[state=open]/collapsible:rotate-180" />
          </CollapsibleTrigger>
        </SidebarGroupLabel>
        <CollapsibleContent>{content}</CollapsibleContent>
      </SidebarGroup>
    </Collapsible>
  );
}

function ContextLink({ item, activeUrl, onNavigate }: {
  item: ContextNavigationItem;
  activeUrl?: string;
  onNavigate: () => void;
}): JSX.Element {
  const location = useLocation();
  const itemPath = item.url.split("?")[0];
  const currentUrl = `${location.pathname}${location.search}`;
  const active = activeUrl ? activeUrl === item.url : currentUrl === item.url || location.pathname === itemPath;

  return (
    <SidebarMenuItem>
      <SidebarMenuButton
        asChild
        isActive={active}
        className="h-9 rounded-md px-2 text-muted-foreground hover:bg-background hover:text-foreground data-[active=true]:bg-background data-[active=true]:font-medium data-[active=true]:text-foreground data-[active=true]:shadow-sm"
      >
        <NavLink to={item.url} onClick={onNavigate}>
          <span className={active ? "flex size-6 items-center justify-center rounded-md bg-primary/10 text-primary" : "flex size-6 items-center justify-center rounded-md"}>
            <item.icon className="size-3.5" aria-hidden="true" />
          </span>
          <span className="truncate">{item.title}</span>
        </NavLink>
      </SidebarMenuButton>
    </SidebarMenuItem>
  );
}
