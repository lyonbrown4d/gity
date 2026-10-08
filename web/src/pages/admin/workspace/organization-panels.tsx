import { Badge } from "@/components/ui/badge";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { useI18n } from "@/lib/i18n";
import type { OrganizationMemberView, RepositoryView } from "@/pages/types";
import { ConsoleEmpty, ConsoleLoadingRows, ConsolePanel, StatusDot } from "./console-primitives";

interface OrganizationPanelsProps {
  members: OrganizationMemberView[];
  repos: RepositoryView[];
  isLoading: boolean;
}

export function OrganizationPanels({ members, repos, isLoading }: OrganizationPanelsProps): JSX.Element {
  const { t } = useI18n();

  return (
    <div className="grid gap-4 xl:grid-cols-2">
      <ConsolePanel
        title={t("Members")}
        description={t("Members of the selected organization.")}
        meta={`${members.length} ${t("total")}`}
        bodyClassName="p-0"
      >
        {isLoading ? <div className="p-3"><ConsoleLoadingRows /></div> : (
          <Table>
            <TableHeader>
              <TableRow className="bg-muted/20">
                <TableHead className="h-8 px-3 text-[10px] uppercase tracking-wider">{t("Username")}</TableHead>
                <TableHead className="h-8 text-[10px] uppercase tracking-wider">{t("Role")}</TableHead>
                <TableHead className="h-8 px-3 text-right text-[10px] uppercase tracking-wider">{t("User ID")}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {members.map((member) => (
                <TableRow key={`${member.organization_id}-${member.user_id}`}>
                  <TableCell className="px-3 py-2">
                    <div className="flex items-center gap-2">
                      <StatusDot />
                      <div className="min-w-0">
                        <p className="truncate text-xs font-medium">{member.username}</p>
                        <p className="truncate text-[10px] text-muted-foreground">{member.email}</p>
                      </div>
                    </div>
                  </TableCell>
                  <TableCell className="py-2"><Badge variant="secondary">{member.role}</Badge></TableCell>
                  <TableCell className="max-w-28 truncate px-3 py-2 text-right font-mono text-[10px] text-muted-foreground">
                    {member.user_id}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
        {!isLoading && members.length === 0 ? <ConsoleEmpty>{t("No members found.")}</ConsoleEmpty> : null}
      </ConsolePanel>

      <ConsolePanel
        title={t("Projects")}
        description={t("Projects under the selected organization.")}
        meta={`${repos.length} ${t("projects")}`}
        bodyClassName="p-0"
      >
        {isLoading ? <div className="p-3"><ConsoleLoadingRows /></div> : (
          <Table>
            <TableHeader>
              <TableRow className="bg-muted/20">
                <TableHead className="h-8 px-3 text-[10px] uppercase tracking-wider">{t("Project")}</TableHead>
                <TableHead className="h-8 text-[10px] uppercase tracking-wider">{t("Visibility")}</TableHead>
                <TableHead className="h-8 px-3 text-right text-[10px] uppercase tracking-wider">{t("Default branch")}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {repos.map((repo) => (
                <TableRow key={repo.id}>
                  <TableCell className="px-3 py-2">
                    <p className="text-xs font-medium">{repo.name}</p>
                    <p className="font-mono text-[10px] text-muted-foreground">{repo.key}</p>
                  </TableCell>
                  <TableCell className="py-2"><Badge variant="outline">{repo.visibility}</Badge></TableCell>
                  <TableCell className="px-3 py-2 text-right font-mono text-[10px] text-muted-foreground">
                    {repo.default_branch}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
        {!isLoading && repos.length === 0 ? <ConsoleEmpty>{t("No projects found.")}</ConsoleEmpty> : null}
      </ConsolePanel>
    </div>
  );
}
