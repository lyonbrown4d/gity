import { Pencil, Trash2 } from "lucide-react";
import { ConfirmAction } from "@/components/common/confirm-action";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { useI18n } from "@/lib/i18n";
import { ConsoleLoadingRows, ConsolePanel, StatusDot } from "./console-primitives";
import type { UserAdminController } from "./use-user-admin";

export function UserDirectory({ controller: c }: { controller: UserAdminController }): JSX.Element {
  const { t } = useI18n();

  return (
    <ConsolePanel
      title={t("User Directory")}
      description={t("Identity, status, and privilege level.")}
      meta={`${c.totalUsers} ${t("total")}`}
      bodyClassName="p-0"
    >
      {c.isLoading ? <div className="p-3"><ConsoleLoadingRows rows={4} /></div> : (
        <Table>
          <TableHeader>
            <TableRow className="bg-muted/20">
              <TableHead className="h-8 px-3 text-[10px] uppercase tracking-wider">{t("Username")}</TableHead>
              <TableHead className="h-8 text-[10px] uppercase tracking-wider">{t("Email")}</TableHead>
              <TableHead className="h-8 text-[10px] uppercase tracking-wider">{t("Status")}</TableHead>
              <TableHead className="h-8 text-[10px] uppercase tracking-wider">{t("Role")}</TableHead>
              <TableHead className="h-8 px-3 text-right text-[10px] uppercase tracking-wider">{t("Actions")}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {c.users.map((user) => (
              <TableRow key={user.id}>
                <TableCell className="px-3 py-2">
                  <div className="flex items-center gap-2">
                    <StatusDot active={user.status.toLowerCase() === "active"} />
                    <div className="min-w-0">
                      <p className="truncate text-xs font-medium">{user.username}</p>
                      <p className="max-w-40 truncate font-mono text-[10px] text-muted-foreground">{user.id}</p>
                    </div>
                  </div>
                </TableCell>
                <TableCell className="max-w-56 truncate py-2 text-xs">{user.email}</TableCell>
                <TableCell className="py-2"><Badge variant="secondary">{user.status}</Badge></TableCell>
                <TableCell className="py-2">
                  <Badge variant={user.is_super_admin ? "default" : "outline"}>
                    {user.is_super_admin ? t("super-admin") : "user"}
                  </Badge>
                </TableCell>
                <TableCell className="px-3 py-2">
                  <div className="flex justify-end gap-1">
                    <Button
                      type="button"
                      size="icon"
                      variant="ghost"
                      className="size-7"
                      onClick={() => c.openEdit(user)}
                      aria-label={`${t("Edit")} ${user.username}`}
                      title={t("Edit")}
                    >
                      <Pencil className="size-3.5" />
                    </Button>
                    <ConfirmAction
                      title={t("Delete user \"{name}\"?").replace("{name}", user.username)}
                      description={t("This action cannot be undone.")}
                      confirmLabel={t("Delete")}
                      cancelLabel={t("Cancel")}
                      onConfirm={() => c.submitDelete(user)}
                    >
                      <Button
                        type="button"
                        size="icon"
                        variant="ghost"
                        className="size-7 text-destructive hover:text-destructive"
                        disabled={c.isDeleting || user.is_super_admin}
                        aria-label={`${t("Delete")} ${user.username}`}
                        title={t("Delete")}
                      >
                        <Trash2 className="size-3.5" />
                      </Button>
                    </ConfirmAction>
                  </div>
                </TableCell>
              </TableRow>
            ))}
            {c.users.length === 0 ? (
              <TableRow>
                <TableCell colSpan={5} className="h-28 text-center text-xs text-muted-foreground">
                  {t("No users found.")}
                </TableCell>
              </TableRow>
            ) : null}
          </TableBody>
        </Table>
      )}
      <div className="flex flex-col gap-2 border-t border-border/70 bg-muted/10 px-3 py-2 sm:flex-row sm:items-center sm:justify-between">
        <p className="font-mono text-[10px] text-muted-foreground">
          {t("Page")} {c.currentPage} / {Math.max(c.pageCount, 1)}
        </p>
        <div className="flex items-center gap-2">
          <Label htmlFor="users-page-size" className="text-[10px] text-muted-foreground">{t("Page size")}</Label>
          <Select value={String(c.pageSize)} onValueChange={(value) => c.setPageSize(Number(value))}>
            <SelectTrigger id="users-page-size" className="h-7 w-20 text-xs"><SelectValue /></SelectTrigger>
            <SelectContent>
              {[10, 20, 50, 100].map((value) => <SelectItem key={value} value={String(value)}>{value}</SelectItem>)}
            </SelectContent>
          </Select>
          <Button type="button" variant="outline" size="sm" className="h-7" onClick={c.previousPage} disabled={c.currentPage <= 1}>
            {t("Prev")}
          </Button>
          <Button type="button" variant="outline" size="sm" className="h-7" onClick={c.nextPage} disabled={c.currentPage >= c.pageCount}>
            {t("Next")}
          </Button>
        </div>
      </div>
    </ConsolePanel>
  );
}
