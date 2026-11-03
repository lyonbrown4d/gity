import { ChevronsUpDown, LogOut, Shield, User } from "lucide-react";
import { Link } from "react-router";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { SidebarMenu, SidebarMenuButton, SidebarMenuItem } from "@/components/ui/sidebar";
import { useI18n } from "@/lib/i18n";

interface NavUserProps {
  user: { name: string; email: string; isSuperAdmin: boolean };
  onLogout: () => void;
  onNavigate?: () => void;
  compact?: boolean;
}

export const NavUser = ({ user, onLogout, onNavigate, compact = false }: NavUserProps): JSX.Element => {
  const { t } = useI18n();
  const initials = user.name
    .trim()
    .split(/\s+/)
    .slice(0, 2)
    .map((part) => part.charAt(0))
    .join("")
    .toUpperCase() || "GU";

  return (
    <SidebarMenu className={compact ? "w-10" : undefined}>
      <SidebarMenuItem>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <SidebarMenuButton
              size="lg"
              aria-label={`${t("My Account")}: ${user.name}`}
              className={compact
                ? "mx-auto size-9 justify-center p-0 text-sidebar-foreground hover:bg-sidebar-accent data-[state=open]:bg-sidebar-accent"
                : "data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"}
            >
              <Avatar className="size-7 rounded-md">
                <AvatarFallback className="rounded-md bg-sidebar-primary text-[10px] font-semibold text-sidebar-primary-foreground">
                  {initials}
                </AvatarFallback>
              </Avatar>
              {compact ? null : (
                <>
                  <div className="grid flex-1 text-left text-sm leading-tight">
                    <span className="truncate font-semibold">{user.name}</span>
                    <span className="truncate text-xs">{user.email}</span>
                  </div>
                  <ChevronsUpDown className="ml-auto size-4" aria-hidden="true" />
                </>
              )}
            </SidebarMenuButton>
          </DropdownMenuTrigger>
          <DropdownMenuContent
            className={compact ? "w-64 rounded-md" : "w-[--radix-dropdown-menu-trigger-width] min-w-56 rounded-md"}
            side="right"
            align="end"
            sideOffset={8}
          >
            <DropdownMenuLabel className="p-2 font-normal">
              <div className="flex items-center gap-2 text-left text-sm">
                <Avatar className="size-8 rounded-md">
                  <AvatarFallback className="rounded-md bg-primary text-xs font-semibold text-primary-foreground">
                    {initials}
                  </AvatarFallback>
                </Avatar>
                <div className="min-w-0 flex-1 leading-tight">
                  <p className="truncate font-semibold">{user.name}</p>
                  <p className="truncate text-xs text-muted-foreground">{user.email}</p>
                </div>
              </div>
            </DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuItem asChild>
              <Link to="/app/profile" onClick={onNavigate}>
                <User aria-hidden="true" />
                {t("Profile")}
              </Link>
            </DropdownMenuItem>
            {user.isSuperAdmin ? (
              <DropdownMenuItem asChild>
                <Link to="/admin" onClick={onNavigate}>
                  <Shield aria-hidden="true" />
                  {t("Admin Dashboard")}
                </Link>
              </DropdownMenuItem>
            ) : null}
            <DropdownMenuSeparator />
            <DropdownMenuItem
              onSelect={() => {
                onNavigate?.();
                onLogout();
              }}
            >
              <LogOut aria-hidden="true" />
              {t("Logout")}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  );
};
