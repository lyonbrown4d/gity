import { Download, Package, RefreshCw } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { DataList, DataRow } from "@/components/surface/data-list";
import { cn } from "@/lib/utils";
import { formatRelativeTime } from "../issues-utils";
import { formatBytes } from "./file-transfer";
import { PackageEmptyState } from "./primitives";
import type { usePackages } from "./use-packages";

interface Props {
  state: ReturnType<typeof usePackages>;
  canWritePackages: boolean;
  t: (key: string) => string;
}
export const PackageBrowser = ({ state, canWritePackages, t }: Props) => {
  const {
    packages,
    selectedPackage,
    packageDetail,
    isLoadingPackages,
    isLoadingDetail,
    isDownloading,
    setSelectedPackageId,
    loadSelectedPackage,
    downloadFile,
    listError,
    detailError,
  } = state;
  return (
    <div className="grid lg:grid-cols-[300px_minmax(0,1fr)]">
      <div className="min-w-0 border-b border-border lg:border-b-0 lg:border-r">
        {listError ? (
          <p role="alert" className="px-4 py-3 text-sm text-destructive">
            {t("Packages could not be loaded. Try reloading.")}
          </p>
        ) : null}
        {isLoadingPackages ? (
          <p className="px-4 py-3 text-sm text-muted-foreground">{t("Loading packages...")}</p>
        ) : null}
        {!listError && !isLoadingPackages && packages.length === 0 ? (
          <div className="px-4 py-6">
            <PackageEmptyState
              title={t("No packages published yet.")}
              description={
                canWritePackages
                  ? t("Upload the first package file to create a version.")
                  : t("Packages will appear here after maintainers publish artifacts.")
              }
            />
          </div>
        ) : null}
        {packages.length > 0 ? (
          <DataList>
            {packages.map((item) => (
              <DataRow key={item.id} className="p-0">
                <button
                  type="button"
                  className={cn(
                    "flex w-full items-center gap-2 px-4 py-2.5 text-left transition-colors hover:bg-muted/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring",
                    selectedPackage?.id === item.id ? "bg-primary/5" : "",
                  )}
                  aria-pressed={selectedPackage?.id === item.id}
                  onClick={() => setSelectedPackageId(item.id)}
                >
                  <Package className="size-3.5 shrink-0 text-muted-foreground" />
                  <span className="min-w-0 flex-1">
                    <span className="block truncate text-sm font-medium">{item.name}</span>
                    <span className="mt-0.5 block truncate text-xs text-muted-foreground">
                      {item.type} · {item.updated_at ? formatRelativeTime(item.updated_at) : t("No updates yet")}
                    </span>
                  </span>
                </button>
              </DataRow>
            ))}
          </DataList>
        ) : null}
      </div>

      <div className="min-w-0 border-t border-border lg:border-t-0">
        {selectedPackage ? (
          <div className="divide-y divide-border">
            <div className="flex flex-wrap items-start justify-between gap-3 px-4 py-3">
              <div className="flex min-w-0 flex-col gap-1">
                <div className="flex flex-wrap items-center gap-2">
                  <h3 className="truncate text-sm font-semibold">{selectedPackage.name}</h3>
                  <Badge variant="secondary">{selectedPackage.type}</Badge>
                </div>
                <p className="text-xs text-muted-foreground">
                  #{selectedPackage.id}
                  {selectedPackage.updated_at ? ` · ${formatRelativeTime(selectedPackage.updated_at)}` : ""}
                </p>
              </div>
              <Button type="button" size="sm" variant="ghost" onClick={() => void loadSelectedPackage()}>
                <RefreshCw data-icon="inline-start" />
                {t("Reload detail")}
              </Button>
            </div>

            {detailError ? (
              <p role="alert" className="px-4 py-3 text-sm text-destructive">
                {t("Package details could not be loaded. Try reloading.")}
              </p>
            ) : null}
            {isLoadingDetail ? (
              <p className="px-4 py-3 text-sm text-muted-foreground">{t("Loading package detail...")}</p>
            ) : null}
            {!detailError && !isLoadingDetail && packageDetail?.versions.length === 0 ? (
              <div className="px-4 py-3">
                <Alert>
                  <AlertTitle>{t("No package versions found.")}</AlertTitle>
                  <AlertDescription>
                    {t("Upload a package file to create the first version for this package.")}
                  </AlertDescription>
                </Alert>
              </div>
            ) : null}
            {packageDetail?.versions.map((item) => (
              <div key={item.version.id}>
                <div className="flex flex-wrap items-center justify-between gap-2 px-4 py-2.5">
                  <div>
                    <p className="text-sm font-medium">{item.version.version}</p>
                    <p className="text-xs text-muted-foreground">
                      {item.version.updated_at ? formatRelativeTime(item.version.updated_at) : t("No updates yet")}
                    </p>
                  </div>
                  <Badge variant="outline">{item.version.status || "default"}</Badge>
                </div>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>{t("File")}</TableHead>
                      <TableHead>{t("Path")}</TableHead>
                      <TableHead>{t("Size")}</TableHead>
                      <TableHead className="text-right">{t("Actions")}</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {item.files.map((file) => (
                      <TableRow key={file.id}>
                        <TableCell className="font-medium">{file.file_name}</TableCell>
                        <TableCell className="max-w-[240px] truncate text-muted-foreground">
                          {file.file_path ?? "--"}
                        </TableCell>
                        <TableCell>{formatBytes(file.byte_size)}</TableCell>
                        <TableCell className="text-right">
                          <Button
                            type="button"
                            size="sm"
                            variant="outline"
                            disabled={isDownloading}
                            onClick={() => void downloadFile(file)}
                          >
                            <Download data-icon="inline-start" />
                            {t("Download")}
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))}
                    {item.files.length === 0 ? (
                      <TableRow>
                        <TableCell colSpan={4} className="text-sm text-muted-foreground">
                          {t("No files in this version.")}
                        </TableCell>
                      </TableRow>
                    ) : null}
                  </TableBody>
                </Table>
              </div>
            ))}
          </div>
        ) : (
          <div className="px-4 py-10">
            <PackageEmptyState
              title={t("Select or upload a package.")}
              description={
                canWritePackages
                  ? t("Upload a package file or select an existing package to inspect versions.")
                  : t("Select an existing package to inspect versions and files.")
              }
            />
          </div>
        )}
      </div>
    </div>
  );
};
