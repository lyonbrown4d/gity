import { useEffect, useMemo, useState } from "react";
import { useList } from "@refinedev/core";
import { CornerDownLeft, Search } from "lucide-react";
import { useLocation, useNavigate } from "react-router";
import { useGlobalQuickJumpSearch } from "@/components/quick-jump/use-global-quick-jump-search";
import { toGlobalSearchQuickJumpItem } from "@/components/quick-jump/global-search-model";
import {
  buildQuickJumpGroups,
  parseCurrentProjectRoute,
  quickJumpValue,
  type QuickJumpItem,
} from "@/components/quick-jump/quick-jump-model";
import { Button } from "@/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from "@/components/ui/command";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Kbd } from "@/components/ui/kbd";
import { useI18n } from "@/lib/i18n";
import type { RepositoryView } from "@/pages/types";

interface GlobalQuickJumpProps {
  canAccessAdmin: boolean;
}

export const GlobalQuickJump = ({ canAccessAdmin }: GlobalQuickJumpProps): JSX.Element => {
  const { t } = useI18n();
  const location = useLocation();
  const navigate = useNavigate();
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const globalSearch = useGlobalQuickJumpSearch(query, open);
  const projectQuery = useList<RepositoryView>({
    resource: "my-projects",
    pagination: { currentPage: 1, pageSize: 30 },
    meta: { all: true },
    queryOptions: { refetchOnWindowFocus: false, retry: false },
  });
  const projects = projectQuery.result.data ?? [];
  const currentProjectRoute = useMemo(() => parseCurrentProjectRoute(location.pathname), [location.pathname]);
  const groups = useMemo(
    () => buildQuickJumpGroups({ canAccessAdmin, currentProjectRoute, projects, t }),
    [canAccessAdmin, currentProjectRoute, projects, t],
  );
  const globalItems = useMemo(
    () => globalSearch.items
      .map((item) => toGlobalSearchQuickJumpItem(item))
      .filter((item): item is NonNullable<typeof item> => item !== null),
    [globalSearch.items],
  );
  const visibleGroups = useMemo(() => {
    const needle = query.trim().toLocaleLowerCase();
    if (!needle) return groups;
    return groups
      .map((group) => ({ ...group, items: group.items.filter((item) => quickJumpValue(item).toLocaleLowerCase().includes(needle)) }))
      .filter((group) => group.items.length > 0);
  }, [groups, query]);
  const globalGroups = useMemo(() => [
    { id: "global-issues", title: t("Issues"), items: globalItems.filter((item) => item.id.startsWith("global-issue:")) },
    { id: "global-code", title: t("Code"), items: globalItems.filter((item) => item.id.startsWith("global-code:")) },
  ].filter((group) => group.items.length > 0), [globalItems, t]);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setOpen((current) => !current);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  const changeOpen = (nextOpen: boolean) => {
    setOpen(nextOpen);
    if (!nextOpen) setQuery("");
  };
  const openItem = (item: QuickJumpItem) => {
    navigate(item.path);
    changeOpen(false);
  };

  return (
    <>
      <Button
        type="button"
        variant="outline"
        size="sm"
        className="hidden h-8 w-full justify-between bg-muted px-2.5 text-muted-foreground shadow-none md:inline-flex"
        onClick={() => changeOpen(true)}
      >
        <span className="flex min-w-0 items-center gap-2">
          <Search data-icon="inline-start" aria-hidden="true" />
          <span className="truncate">{t("Quick jump")}</span>
        </span>
        <Kbd className="ml-3 shrink-0">Ctrl K</Kbd>
      </Button>
      <Button
        type="button"
        variant="ghost"
        size="icon"
        className="md:hidden"
        aria-label={t("Quick jump")}
        onClick={() => changeOpen(true)}
      >
        <Search aria-hidden="true" />
      </Button>

      <Dialog open={open} onOpenChange={changeOpen}>
        <DialogContent className="max-w-2xl gap-0 overflow-hidden p-0">
          <DialogHeader className="sr-only">
            <DialogTitle>{t("Quick jump")}</DialogTitle>
            <DialogDescription>{t("Search projects, repository sections, and workspace destinations.")}</DialogDescription>
          </DialogHeader>
          <Command loop shouldFilter={false}>
            <CommandInput
              value={query}
              onValueChange={setQuery}
              placeholder={t("Search projects, issues, merge requests, files...")}
              className="pr-8"
            />
            <CommandList className="max-h-[min(60vh,32rem)] px-1 py-2">
              <CommandEmpty>
                {globalSearch.isSearching
                  ? t("Searching globally...")
                  : globalSearch.error
                    ? t("Global search is unavailable. Check your access and try again.")
                    : projectQuery.query.isError
                  ? t("Project shortcuts could not be loaded, but global navigation is still available.")
                  : t("No quick jump results.")}
              </CommandEmpty>
              {globalSearch.isSearching ? <p role="status" className="px-3 py-2 text-xs text-muted-foreground">{t("Searching globally...")}</p> : null}
              {globalSearch.isEmpty ? <p role="status" className="px-3 py-2 text-xs text-muted-foreground">{t("No global code or issue results.")}</p> : null}
              {visibleGroups.map((group, index) => (
                <div key={group.id}>
                  {index > 0 ? <CommandSeparator /> : null}
                  <CommandGroup heading={group.title}>
                    {group.items.map((item) => (
                      <CommandItem
                        key={item.id}
                        value={quickJumpValue(item)}
                        keywords={item.keywords}
                        onSelect={() => openItem(item)}
                        className="items-start py-2.5"
                      >
                        <item.icon aria-hidden="true" />
                        <span className="min-w-0 flex-1">
                          <span className="block truncate font-medium">{item.title}</span>
                          <span className="block truncate text-xs text-muted-foreground">{item.description}</span>
                        </span>
                      </CommandItem>
                    ))}
                  </CommandGroup>
                </div>
              ))}
              {globalGroups.map((group) => (
                <div key={group.id}>
                  <CommandSeparator />
                  <CommandGroup heading={group.title}>
                    {group.items.map((item) => (
                      <CommandItem
                        key={item.id}
                        value={quickJumpValue(item)}
                        keywords={item.keywords}
                        onSelect={() => openItem(item)}
                        className="items-start py-2.5"
                      >
                        <item.icon aria-hidden="true" />
                        <span className="min-w-0 flex-1">
                          <span className="block truncate font-medium">{item.title}</span>
                          <span className="block truncate text-xs text-muted-foreground">{item.description}</span>
                        </span>
                      </CommandItem>
                    ))}
                  </CommandGroup>
                </div>
              ))}
              {globalSearch.error ? <p role="alert" className="px-3 py-2 text-xs text-destructive">{t("Global search is unavailable. Check your access and try again.")}</p> : null}
              {globalSearch.hasMore ? <p role="status" className="px-3 py-2 text-xs text-muted-foreground">{t("More matches available. Refine your search.")}</p> : null}
              {globalSearch.partial && globalGroups.length > 0 ? <p role="status" className="px-3 py-2 text-xs text-muted-foreground">{t("Some results may be missing while the index catches up.")}</p> : null}
            </CommandList>
            <div className="flex items-center justify-between border-t px-3 py-2 text-xs text-muted-foreground">
              <span>{t("Type to search destinations and projects")}</span>
              <span className="flex items-center gap-1.5">
                <Kbd>↑</Kbd>
                <Kbd>↓</Kbd>
                <span>{t("navigate")}</span>
                <Kbd><CornerDownLeft aria-hidden="true" /></Kbd>
                <span>{t("open")}</span>
              </span>
            </div>
          </Command>
        </DialogContent>
      </Dialog>
    </>
  );
};
