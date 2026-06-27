import { useEffect, useMemo, useState } from "react";
import { Copy, Download, Package, Plus, RefreshCw, Terminal } from "lucide-react";
import { useCustom, useCustomMutation, useDataProvider } from "@refinedev/core";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Textarea } from "@/components/ui/textarea";
import { getApiBaseUrl } from "@/lib/api";
import { cn } from "@/lib/utils";
import type {
  RepositoryPackageDetailView,
  RepositoryPackageFileContentView,
  RepositoryPackageFileView,
  RepositoryPackageVersionDetailView,
  RepositoryPackageVersionView,
  RepositoryPackageView,
} from "@/pages/types";
import { extractErrorMessage, formatRelativeTime } from "./issues-utils";
import type { RepositoryPermissions } from "./repository-permissions";
import { isRecord, normalizeNumber, normalizeOptionalString, normalizeString, resolveBody, resolveRecordArray, type RawRecord } from "./repository-normalizers";

interface RepositoryPackagesTabProps {
  repoId: string;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  onError: (message: string | null) => void;
}

type RawPackage = RawRecord;

export const RepositoryPackagesTab = ({ repoId, permissions, t, onError }: RepositoryPackagesTabProps): JSX.Element => {
  const dataProvider = useDataProvider();
  const packagesQuery = useCustom<RawPackage[]>({
    url: `/projects/${repoId}/packages`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const [selectedPackageId, setSelectedPackageId] = useState<string | null>(null);
  const packageDetailQuery = useCustom<RawPackage>({
    url: selectedPackageId ? `/projects/${repoId}/packages/${selectedPackageId}` : `/projects/${repoId}/packages/0`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId && selectedPackageId),
      refetchOnWindowFocus: false,
    },
  });
  const { mutateAsync: uploadPackageFile, mutation: { isPending: isUploading } } = useCustomMutation<RawPackage>();
  const [isDownloading, setDownloading] = useState(false);
  const [isComposerOpen, setComposerOpen] = useState(false);
  const [packageType, setPackageType] = useState("generic");
  const [packageName, setPackageName] = useState("");
  const [version, setVersion] = useState("0.1.0");
  const [fileName, setFileName] = useState("artifact.txt");
  const [filePath, setFilePath] = useState("");
  const [contentType, setContentType] = useState("text/plain");
  const [content, setContent] = useState("");
  const [fileContentBase64, setFileContentBase64] = useState("");
  const [copiedCommand, setCopiedCommand] = useState<string | null>(null);

  const packages = useMemo(
    () => resolvePackageList(packagesQuery.result.data).map(normalizePackage),
    [packagesQuery.result.data],
  );
  const selectedPackage = useMemo(
    () => packages.find((item) => item.id === selectedPackageId) ?? packages[0] ?? null,
    [packages, selectedPackageId],
  );
  const packageDetail = useMemo(
    () => normalizePackageDetail(packageDetailQuery.result.data),
    [packageDetailQuery.result.data],
  );
  const totalFiles = packageDetail?.versions.reduce((total, item) => total + item.files.length, 0) ?? 0;
  const selectedVersion = packageDetail?.versions[0] ?? null;
  const selectedFile = selectedVersion?.files[0] ?? null;
  const protocolGuides = useMemo(
    () => buildPackageProtocolGuides(repoId, selectedPackage, selectedVersion, selectedFile),
    [repoId, selectedPackage, selectedVersion, selectedFile],
  );
  const isLoadingPackages = packagesQuery.query.isFetching && !packagesQuery.query.data;
  const isLoadingDetail = packageDetailQuery.query.isFetching && !packageDetailQuery.query.data;
  const canWritePackages = permissions.packageWrite;

  const loadPackages = async () => {
    const result = await packagesQuery.query.refetch();
    if (result.error) {
      onError(extractErrorMessage(result.error));
      return;
    }
    onError(null);
  };

  const loadSelectedPackage = async () => {
    const result = await packageDetailQuery.query.refetch();
    if (result.error) {
      onError(extractErrorMessage(result.error));
      return;
    }
    onError(null);
  };

  const submitUpload = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const normalizedName = packageName.trim();
    const normalizedVersion = version.trim();
    const normalizedFileName = fileName.trim();
    const encodedContent = fileContentBase64 || encodeBase64(content);
    if (!normalizedName || !normalizedVersion || !normalizedFileName || !encodedContent) {
      onError(t("Package name, version, file name, and content are required"));
      return;
    }

    onError(null);
    try {
      const response = await uploadPackageFile({
        url: `/projects/${repoId}/packages/files`,
        method: "post",
        values: {
          type: packageType,
          name: normalizedName,
          version: normalizedVersion,
          file_name: normalizedFileName,
          file_path: filePath.trim() || normalizedFileName,
          content_type: contentType.trim(),
          content_base64: encodedContent,
        },
      });
      const uploaded = normalizePackageFile(resolveBody(response.data));
      setSelectedPackageId(null);
      setComposerOpen(false);
      setFileName("artifact.txt");
      setFilePath("");
      setContent("");
      setFileContentBase64("");
      await loadPackages();
      if (selectedPackage?.id) {
        await loadSelectedPackage();
      }
      if (uploaded.id) {
        onError(null);
      }
    } catch (error) {
      onError(extractErrorMessage(error));
    }
  };

  const downloadFile = async (file: RepositoryPackageFileView) => {
    const custom = dataProvider().custom;
    if (!custom) {
      onError(t("Package file download is not available."));
      return;
    }
    onError(null);
    setDownloading(true);
    try {
      const response = await custom<RawPackage>({
        url: `/projects/${repoId}/packages/files/${file.id}`,
        method: "get",
      });
      const payload = normalizePackageFileContent(response.data);
      saveBase64File(payload.content_base64, payload.file.file_name || file.file_name, payload.file.content_type ?? file.content_type ?? "application/octet-stream");
    } catch (error) {
      onError(extractErrorMessage(error));
    } finally {
      setDownloading(false);
    }
  };

  const copyCommand = async (command: string) => {
    try {
      await navigator.clipboard.writeText(command);
      setCopiedCommand(command);
      window.setTimeout(() => setCopiedCommand((current) => (current === command ? null : current)), 1400);
      onError(null);
    } catch {
      onError(t("Failed to copy command."));
    }
  };

  const selectPackageFile = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) {
      setFileContentBase64("");
      return;
    }
    setFileName(file.name);
    setContentType(file.type || "application/octet-stream");
    if (!filePath.trim()) {
      setFilePath(file.name);
    }
    setFileContentBase64(await readFileAsBase64(file));
    setContent("");
  };

  useEffect(() => {
    if (!repoId) {
      return;
    }
    onError(null);
  }, [repoId, onError]);

  useEffect(() => {
    if (!packagesQuery.query.error) {
      return;
    }
    onError(extractErrorMessage(packagesQuery.query.error));
  }, [packagesQuery.query.error, onError]);

  useEffect(() => {
    if (!packageDetailQuery.query.error) {
      return;
    }
    onError(extractErrorMessage(packageDetailQuery.query.error));
  }, [packageDetailQuery.query.error, onError]);

  useEffect(() => {
    if (packages.length === 0) {
      setSelectedPackageId(null);
      return;
    }
    if (!selectedPackageId || !packages.some((item) => item.id === selectedPackageId)) {
      setSelectedPackageId(packages[0].id);
    }
  }, [packages, selectedPackageId]);

  return (
    <Card className="card-enter">
      <CardHeader>
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div className="flex flex-col gap-1.5">
            <CardTitle>{t("Package Registry")}</CardTitle>
            <CardDescription>{t("Publish generic project artifacts and inspect package versions.")}</CardDescription>
          </div>
          <Badge variant={canWritePackages ? "secondary" : "outline"}>
            {canWritePackages ? t("Upload enabled") : t("Read only")}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <div className="grid gap-3 sm:grid-cols-3">
          <PackageStat label={t("Packages")} value={packages.length} />
          <PackageStat label={t("Versions")} value={packageDetail?.versions.length ?? 0} />
          <PackageStat label={t("Files")} value={totalFiles} />
        </div>

        <div className="flex flex-wrap items-center justify-between gap-2">
          <Button
            type="button"
            variant={isComposerOpen ? "secondary" : "default"}
            disabled={!canWritePackages}
            onClick={() => setComposerOpen((current) => !current)}
          >
            <Plus data-icon="inline-start" />
            {isComposerOpen ? t("Hide upload form") : t("Upload package file")}
          </Button>
          <Button type="button" size="sm" variant="ghost" onClick={() => void loadPackages()}>
            <RefreshCw data-icon="inline-start" />
            {t("Reload")}
          </Button>
        </div>

        {!canWritePackages ? (
          <Alert>
            <AlertTitle>{t("Package registry is read-only")}</AlertTitle>
            <AlertDescription>{t("Your current project role can inspect packages, but cannot upload them.")}</AlertDescription>
          </Alert>
        ) : null}

        {isComposerOpen ? (
          <form className="flex flex-col gap-3 rounded-md border p-3" onSubmit={submitUpload}>
            <div className="grid gap-3 md:grid-cols-[180px_1fr_180px]">
              <div className="flex flex-col gap-1">
                <Label htmlFor="package-type">{t("Package type")}</Label>
                <Select value={packageType} onValueChange={setPackageType}>
                  <SelectTrigger id="package-type">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      <SelectItem value="generic">generic</SelectItem>
                      <SelectItem value="npm">npm</SelectItem>
                      <SelectItem value="maven">maven</SelectItem>
                      <SelectItem value="container">container</SelectItem>
                    </SelectGroup>
                  </SelectContent>
                </Select>
              </div>
              <div className="flex flex-col gap-1">
                <Label htmlFor="package-name">{t("Package name")}</Label>
                <Input
                  id="package-name"
                  value={packageName}
                  onChange={(event) => setPackageName(event.target.value)}
                  placeholder="gity-cli"
                />
              </div>
              <div className="flex flex-col gap-1">
                <Label htmlFor="package-version">{t("Version")}</Label>
                <Input
                  id="package-version"
                  value={version}
                  onChange={(event) => setVersion(event.target.value)}
                  placeholder="0.1.0"
                />
              </div>
            </div>
            <div className="grid gap-3 md:grid-cols-[1fr_1fr_200px]">
              <div className="flex flex-col gap-1">
                <Label htmlFor="package-file-name">{t("File name")}</Label>
                <Input
                  id="package-file-name"
                  value={fileName}
                  onChange={(event) => setFileName(event.target.value)}
                  placeholder="artifact.tar.gz"
                />
              </div>
              <div className="flex flex-col gap-1">
                <Label htmlFor="package-file-path">{t("File path optional")}</Label>
                <Input
                  id="package-file-path"
                  value={filePath}
                  onChange={(event) => setFilePath(event.target.value)}
                  placeholder="dist/artifact.tar.gz"
                />
              </div>
              <div className="flex flex-col gap-1">
                <Label htmlFor="package-content-type">{t("Content type")}</Label>
                <Input
                  id="package-content-type"
                  value={contentType}
                  onChange={(event) => setContentType(event.target.value)}
                  placeholder="application/octet-stream"
                />
              </div>
            </div>
            <div className="flex flex-col gap-1">
              <Label htmlFor="package-file">{t("Package file")}</Label>
              <Input id="package-file" type="file" onChange={(event) => void selectPackageFile(event)} />
              {fileContentBase64 ? (
                <p className="text-xs text-muted-foreground">{t("Selected file content will be uploaded as base64.")}</p>
              ) : null}
            </div>
            <div className="flex flex-col gap-1">
              <Label htmlFor="package-content">{t("Inline file content fallback")}</Label>
              <Textarea
                id="package-content"
                className="min-h-32"
                value={content}
                onChange={(event) => setContent(event.target.value)}
                placeholder="Artifact content for local smoke testing"
              />
            </div>
            <div className="flex justify-end">
              <Button type="submit" disabled={!canWritePackages || isUploading}>
                {isUploading ? t("Uploading package file...") : t("Upload file")}
              </Button>
            </div>
          </form>
        ) : null}

        <div className="grid gap-4 lg:grid-cols-[300px_1fr]">
          <div className="flex flex-col gap-2 rounded-md border p-2">
            {isLoadingPackages ? <p className="px-2 py-2 text-sm text-muted-foreground">{t("Loading packages...")}</p> : null}
            {!isLoadingPackages && packages.length === 0 ? (
              <PackageEmptyState
                title={t("No packages published yet.")}
                description={canWritePackages ? t("Upload the first package file to create a version.") : t("Packages will appear here after maintainers publish artifacts.")}
              />
            ) : null}
            {packages.map((item) => (
              <button
                key={item.id}
                type="button"
                className={cn(
                  "flex w-full flex-col gap-2 rounded-md border p-3 text-left transition hover:bg-muted/40",
                  selectedPackage?.id === item.id ? "border-primary bg-primary/5" : "",
                )}
                onClick={() => setSelectedPackageId(item.id)}
              >
                <div className="flex items-start gap-2">
                  <Package className="mt-0.5 size-4 text-muted-foreground" />
                  <div className="min-w-0">
                    <p className="truncate font-medium">{item.name}</p>
                    <p className="text-xs text-muted-foreground">{item.type}</p>
                  </div>
                </div>
                <p className="text-xs text-muted-foreground">
                  {item.updated_at ? formatRelativeTime(item.updated_at) : t("No updates yet")}
                </p>
              </button>
            ))}
          </div>

          <div className="rounded-md border">
            {selectedPackage ? (
              <div className="flex flex-col gap-4 p-4">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="flex min-w-0 flex-col gap-1">
                    <div className="flex flex-wrap items-center gap-2">
                      <h3 className="truncate text-lg font-semibold">{selectedPackage.name}</h3>
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

                {isLoadingDetail ? <p className="text-sm text-muted-foreground">{t("Loading package detail...")}</p> : null}
                {!isLoadingDetail && packageDetail?.versions.length === 0 ? (
                  <Alert>
                    <AlertTitle>{t("No package versions found.")}</AlertTitle>
                    <AlertDescription>{t("Upload a package file to create the first version for this package.")}</AlertDescription>
                  </Alert>
                ) : null}
                {packageDetail?.versions.map((item) => (
                  <div key={item.version.id} className="flex flex-col gap-2 rounded-md border p-3">
                    <div className="flex flex-wrap items-center justify-between gap-2">
                      <div>
                        <p className="font-medium">{item.version.version}</p>
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
                            <TableCell className="max-w-[240px] truncate text-muted-foreground">{file.file_path ?? "--"}</TableCell>
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
              <PackageEmptyState
                className="min-h-64 border-0"
                title={t("Select or upload a package.")}
                description={canWritePackages ? t("Upload a package file or select an existing package to inspect versions.") : t("Select an existing package to inspect versions and files.")}
              />
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
};

interface PackageProtocolGuideItem {
  id: string;
  title: string;
  description: string;
  endpoint: string;
  scopes: string[];
  publishCommands: string[];
  installCommands: string[];
}

const PackageProtocolGuide = ({
  guides,
  copiedCommand,
  t,
  onCopy,
}: {
  guides: PackageProtocolGuideItem[];
  copiedCommand: string | null;
  t: (text: string) => string;
  onCopy: (command: string) => void;
}) => {
  const [activeProtocol, setActiveProtocol] = useState("generic");
  const activeGuide = guides.find((guide) => guide.id === activeProtocol) ?? guides[0];

  useEffect(() => {
    if (!guides.some((guide) => guide.id === activeProtocol)) {
      setActiveProtocol(guides[0]?.id ?? "generic");
    }
  }, [activeProtocol, guides]);

  if (!activeGuide) {
    return null;
  }

  return (
    <div className="grid gap-3 rounded-xl border border-border/80 bg-muted/20 p-3 lg:grid-cols-[240px_1fr]">
      <div className="flex flex-col gap-2">
        <div className="flex items-center gap-2">
          <Terminal className="size-4 text-primary" />
          <p className="font-medium">{t("Registry setup guide")}</p>
        </div>
        <p className="text-xs leading-5 text-muted-foreground">
          {t("Copy publish and install commands for the selected package protocol. Use a project access token or deploy token with package scopes.")}
        </p>
        <Select value={activeProtocol} onValueChange={setActiveProtocol}>
          <SelectTrigger aria-label={t("Package protocol")}>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              {guides.map((guide) => (
                <SelectItem key={guide.id} value={guide.id}>{guide.title}</SelectItem>
              ))}
            </SelectGroup>
          </SelectContent>
        </Select>
        <div className="flex flex-wrap gap-1">
          {activeGuide.scopes.map((scope) => (
            <Badge key={scope} variant="outline">{scope}</Badge>
          ))}
        </div>
      </div>
      <div className="grid gap-3 xl:grid-cols-2">
        <div className="flex flex-col gap-2 rounded-lg border bg-background/80 p-3">
          <div>
            <p className="text-sm font-medium">{activeGuide.title}</p>
            <p className="text-xs text-muted-foreground">{t(activeGuide.description)}</p>
          </div>
          <PackageCommandLine label={t("Endpoint")} value={activeGuide.endpoint} copiedCommand={copiedCommand} t={t} onCopy={onCopy} />
          {activeGuide.publishCommands.map((command) => (
            <PackageCommandLine key={command} label={t("Publish")} value={command} copiedCommand={copiedCommand} t={t} onCopy={onCopy} />
          ))}
        </div>
        <div className="flex flex-col gap-2 rounded-lg border bg-background/80 p-3">
          <p className="text-sm font-medium">{t("Install or consume")}</p>
          {activeGuide.installCommands.map((command) => (
            <PackageCommandLine key={command} label={t("Install")} value={command} copiedCommand={copiedCommand} t={t} onCopy={onCopy} />
          ))}
        </div>
      </div>
    </div>
  );
};

const PackageCommandLine = ({
  label,
  value,
  copiedCommand,
  t,
  onCopy,
}: {
  label: string;
  value: string;
  copiedCommand: string | null;
  t: (text: string) => string;
  onCopy: (command: string) => void;
}) => (
  <div className="flex flex-col gap-1">
    <div className="flex items-center justify-between gap-2">
      <span className="text-xs font-medium text-muted-foreground">{label}</span>
      <Button type="button" size="sm" variant="ghost" className="h-7 px-2" onClick={() => onCopy(value)}>
        <Copy className="size-3.5" />
        {copiedCommand === value ? t("Copied") : t("Copy")}
      </Button>
    </div>
    <code className="block overflow-x-auto rounded-md bg-muted px-3 py-2 font-mono text-xs leading-5 text-foreground">
      {value}
    </code>
  </div>
);
const PackageStat = ({ label, value }: { label: string; value: number }) => (
  <div className="flex flex-col gap-1 rounded-md border p-3">
    <p className="text-xs text-muted-foreground">{label}</p>
    <p className="text-lg font-semibold">{value}</p>
  </div>
);

const PackageEmptyState = ({ title, description, className }: { title: string; description: string; className?: string }) => (
  <div className={cn("flex flex-col items-center justify-center gap-2 rounded-md border border-dashed p-4 text-center", className)}>
    <Package className="size-5 text-muted-foreground" />
    <div className="flex flex-col gap-1">
      <p className="text-sm font-medium">{title}</p>
      <p className="text-xs text-muted-foreground">{description}</p>
    </div>
  </div>
);

const resolvePackageList = (payload: unknown): RawPackage[] => {
  const raw = resolveBody(payload);
  return resolveRecordArray(raw);
};

const normalizePackageDetail = (payload: unknown): RepositoryPackageDetailView | null => {
  const raw = resolveBody(payload);
  if (!isRecord(raw)) {
    return null;
  }
  const versionsRaw = raw.versions ?? raw.Versions;
  return {
    package: normalizePackage(raw.package ?? raw.Package),
    versions: resolveRecordArray(versionsRaw).map(normalizePackageVersionDetail),
  };
};

const normalizePackageVersionDetail = (raw: RawPackage): RepositoryPackageVersionDetailView => {
  const files = raw.files ?? raw.Files;
  return {
    version: normalizePackageVersion(raw.version ?? raw.Version),
    files: resolveRecordArray(files).map(normalizePackageFile),
  };
};

const normalizePackageFileContent = (payload: unknown): RepositoryPackageFileContentView => {
  const raw = resolveBody(payload);
  if (!isRecord(raw)) {
    return { file: normalizePackageFile({}), content_base64: "" };
  }
  return {
    file: normalizePackageFile(raw.file ?? raw.File),
    content_base64: normalizeString(raw.content_base64 ?? raw.ContentBase64 ?? raw.Content),
  };
};

const normalizePackage = (value: unknown): RepositoryPackageView => {
  const raw = isRecord(value) ? value : {};
  return {
    id: normalizeString(raw.id ?? raw.ID),
    project_id: normalizeString(raw.project_id ?? raw.ProjectID),
    type: normalizeString(raw.type ?? raw.Type) || "generic",
    name: normalizeString(raw.name ?? raw.Name) || "package",
    created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
    updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
  };
};

const normalizePackageVersion = (value: unknown): RepositoryPackageVersionView => {
  const raw = isRecord(value) ? value : {};
  return {
    id: normalizeString(raw.id ?? raw.ID),
    project_package_id: normalizeString(raw.project_package_id ?? raw.ProjectPackageID),
    version: normalizeString(raw.version ?? raw.Version) || "0.0.0",
    status: normalizeString(raw.status ?? raw.Status) || "default",
    created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
    updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
  };
};

const normalizePackageFile = (value: unknown): RepositoryPackageFileView => {
  const raw = isRecord(value) ? value : {};
  return {
    id: normalizeString(raw.id ?? raw.ID),
    project_package_version_id: normalizeString(raw.project_package_version_id ?? raw.ProjectPackageVersionID),
    file_name: normalizeString(raw.file_name ?? raw.FileName) || "artifact",
    file_path: normalizeOptionalString(raw.file_path ?? raw.FilePath),
    content_type: normalizeOptionalString(raw.content_type ?? raw.ContentType),
    byte_size: normalizeNumber(raw.byte_size ?? raw.ByteSize),
    created_at: normalizeOptionalString(raw.created_at ?? raw.CreatedAt),
    updated_at: normalizeOptionalString(raw.updated_at ?? raw.UpdatedAt),
  };
};

const encodeBase64 = (value: string): string => {
  const bytes = new TextEncoder().encode(value);
  let binary = "";
  bytes.forEach((byte) => {
    binary += String.fromCharCode(byte);
  });
  return window.btoa(binary);
};

const readFileAsBase64 = (file: File): Promise<string> =>
  new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.addEventListener("load", () => {
      const result = normalizeString(reader.result);
      const [, content = ""] = result.split(",", 2);
      resolve(content);
    });
    reader.addEventListener("error", () => reject(reader.error ?? new Error("failed to read file")));
    reader.readAsDataURL(file);
  });

const saveBase64File = (contentBase64: string, fileName: string, contentType: string) => {
  const binary = window.atob(contentBase64);
  const bytes = Uint8Array.from(binary, (character) => character.charCodeAt(0));
  const blob = new Blob([bytes], { type: contentType });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = fileName;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
};

const formatBytes = (value: number): string => {
  if (value < 1024) {
    return `${value} B`;
  }
  if (value < 1024 * 1024) {
    return `${(value / 1024).toFixed(1)} KiB`;
  }
  return `${(value / 1024 / 1024).toFixed(1)} MiB`;
};

const buildPackageProtocolGuides = (
  repoId: string,
  selectedPackage: RepositoryPackageView | null,
  selectedVersion: RepositoryPackageVersionDetailView | null,
  selectedFile: RepositoryPackageFileView | null,
): PackageProtocolGuideItem[] => {
  const packageName = encodePathSegment(selectedPackage?.name || "my-package");
  const versionValue = encodePathSegment(selectedVersion?.version.version || "0.1.0");
  const filePath = encodePathTail(selectedFile?.file_path || selectedFile?.file_name || "artifact.tar.gz");
  const apiBase = absoluteApiUrl(`/projects/${repoId}/packages`);
  const genericEndpoint = `${apiBase}/generic/${packageName}/${versionValue}/${filePath}`;
  const nugetEndpoint = `${apiBase}/nuget/index.json`;
  const mavenFilePath = "com/example/demo/0.1.0/demo-0.1.0.jar";
  const pypiPackageName = encodePathSegment(selectedPackage?.name || "my_package");
  const npmPackageName = selectedPackage?.name?.startsWith("@") ? selectedPackage.name : "@gity/my-package";
  const npmEncodedPackage = encodePathTail(npmPackageName);

  return [
    {
      id: "generic",
      title: "Generic",
      description: "Upload arbitrary build artifacts through the generic registry endpoint.",
      endpoint: genericEndpoint,
      scopes: ["read_package", "write_package"],
      publishCommands: [`curl --request PUT --header "Authorization: Bearer <token>" --upload-file ./artifact.tar.gz ${genericEndpoint}`],
      installCommands: [`curl --location --header "Authorization: Bearer <token>" --output artifact.tar.gz ${genericEndpoint}`],
    },
    {
      id: "npm",
      title: "npm",
      description: "Use the npm endpoint for package metadata and publish payloads.",
      endpoint: `${apiBase}/npm/${npmEncodedPackage}`,
      scopes: ["read_package", "write_package"],
      publishCommands: ["npm publish --registry " + apiBase + "/npm/"],
      installCommands: [
        "npm config set @gity:registry " + apiBase + "/npm/",
        "npm install " + npmPackageName,
      ],
    },
    {
      id: "maven",
      title: "Maven",
      description: "Upload Maven artifacts by repository-relative artifact path.",
      endpoint: `${apiBase}/maven/${mavenFilePath}`,
      scopes: ["read_package", "write_package"],
      publishCommands: [`curl --request PUT --header "Authorization: Bearer <token>" --upload-file ./demo-0.1.0.jar ${apiBase}/maven/${mavenFilePath}`],
      installCommands: [`mvn dependency:get -DremoteRepositories=gity::::${apiBase}/maven -Dartifact=com.example:demo:0.1.0`],
    },
    {
      id: "pypi",
      title: "PyPI",
      description: "Expose a simple Python package index and upload distribution files.",
      endpoint: `${apiBase}/pypi/simple`,
      scopes: ["read_package", "write_package"],
      publishCommands: [`curl --request PUT --header "Authorization: Bearer <token>" --upload-file ./dist/my_package-0.1.0.tar.gz ${apiBase}/pypi/${pypiPackageName}/${versionValue}/my_package-0.1.0.tar.gz`],
      installCommands: [`pip install --index-url ${apiBase}/pypi/simple ${selectedPackage?.name || "my-package"}`],
    },
    {
      id: "nuget",
      title: "NuGet",
      description: "Use the NuGet v3 service index and raw package upload endpoint.",
      endpoint: nugetEndpoint,
      scopes: ["read_package", "write_package"],
      publishCommands: [`curl --request PUT --header "Authorization: Bearer <token>" --upload-file ./My.Package.0.1.0.nupkg ${apiBase}/nuget/My.Package/0.1.0/My.Package.0.1.0.nupkg`],
      installCommands: [
        `dotnet nuget add source ${nugetEndpoint} --name gity --username <username> --password <token> --store-password-in-clear-text`,
        "dotnet add package My.Package --version 0.1.0 --source gity",
      ],
    },
  ];
};

const absoluteApiUrl = (path: string): string => {
  const base = getApiBaseUrl().replace(/\/+$/, "");
  const normalizedPath = path.startsWith("/") ? path : `/${path}`;
  if (/^https?:\/\//i.test(base)) {
    return `${base}${normalizedPath}`;
  }
  if (typeof window === "undefined") {
    return `${base}${normalizedPath}`;
  }
  return `${window.location.origin}${base}${normalizedPath}`;
};

const encodePathSegment = (value: string): string => encodeURIComponent(value).replace(/%2F/gi, "/");

const encodePathTail = (value: string): string => value.split("/").map((part) => encodeURIComponent(part)).join("/");
