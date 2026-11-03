import { RefreshCw } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Section } from "@/components/surface/section";
import { Toolbar } from "@/components/surface/toolbar";
import type { RepositoryPermissions } from "./repository-permissions";
import { PackageBrowser } from "./packages/package-browser";
import { PackageStat } from "./packages/primitives";
import { PackageProtocolGuide } from "./packages/protocol-guide";
import { PackageUploadDialog } from "./packages/upload-dialog";
import { usePackages } from "./packages/use-packages";

interface RepositoryPackagesTabProps {
  repoId: string;
  permissions: RepositoryPermissions;
  t: (text: string) => string;
  onError: (message: string | null) => void;
}

export const RepositoryPackagesTab = ({ repoId, permissions, t, onError }: RepositoryPackagesTabProps): JSX.Element => {
  const state = usePackages(repoId, t, onError);
  const canWritePackages = permissions.packageWrite;
  return <Section title={t("Package Registry")}
    description={t("Publish generic project artifacts and inspect package versions.")}
    actions={<Toolbar>
      <Badge variant={canWritePackages ? "secondary" : "outline"}>{canWritePackages ? t("Upload enabled") : t("Read only")}</Badge>
      <PackageUploadDialog repoId={repoId} canUpload={canWritePackages} t={t} onUploaded={state.uploaded} />
      <Button type="button" size="sm" variant="ghost" onClick={() => void state.loadPackages()}><RefreshCw data-icon="inline-start" />{t("Reload")}</Button>
    </Toolbar>} flush
  >
    <div className="grid divide-y divide-border border-b border-border sm:grid-cols-3 sm:divide-x sm:divide-y-0">
      <PackageStat label={t("Packages")} value={state.packages.length} />
      <PackageStat label={t("Versions")} value={state.packageDetail?.versions.length ?? 0} />
      <PackageStat label={t("Files")} value={state.totalFiles} />
    </div>
    {!canWritePackages && <div className="border-b border-border px-4 py-3"><Alert><AlertTitle>{t("Package registry is read-only")}</AlertTitle><AlertDescription>{t("Your current project role can inspect packages, but cannot upload them.")}</AlertDescription></Alert></div>}
    <PackageBrowser state={state} canWritePackages={canWritePackages} t={t} />
    <details className="border-t border-border p-4">
      <summary className="cursor-pointer text-sm font-medium">{t("Registry setup guide")}</summary>
      <div className="mt-4"><PackageProtocolGuide guides={state.protocolGuides} copiedCommand={state.copiedCommand} t={t} onCopy={(command) => void state.copyCommand(command)} /></div>
    </details>
  </Section>;
};
