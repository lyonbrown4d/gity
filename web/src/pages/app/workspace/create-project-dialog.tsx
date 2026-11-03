import { FormDialog } from "@/components/common/form-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import type { GitignoreTemplate } from "@/lib/generated/model/gitignoreTemplate";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useI18n } from "@/lib/i18n";
import type { LicenseTemplate } from "@/lib/generated/model/licenseTemplate";
import type { OrganizationView } from "@/pages/types";

export interface ProjectDraft {
  ownerOrganizationId: string;
  key: string;
  name: string;
  description: string;
  defaultBranch: string;
  visibility: string;
  gitignoreTemplate: string;
  licenseTemplate: string;
  initializeWithReadme: boolean;
  licenseCopyrightHolder: string;
}

export const createProjectDraft = (ownerOrganizationId = ""): ProjectDraft => ({
  ownerOrganizationId,
  key: "",
  name: "",
  description: "",
  defaultBranch: "main",
  visibility: "private",
  gitignoreTemplate: "none",
  licenseTemplate: "none",
  initializeWithReadme: false,
  licenseCopyrightHolder: "",
});

interface CreateProjectDialogProps {
  open: boolean;
  organizations: OrganizationView[];
  draft: ProjectDraft;
  isSubmitting: boolean;
  licenseTemplates: LicenseTemplate[];
  gitignoreTemplates: GitignoreTemplate[];
  isLoadingLicenses: boolean;
  isLoadingGitignores: boolean;
  licenseError: string | null;
  gitignoreError: string | null;
  onRetryLicenses: () => void;
  onRetryGitignores: () => void;
  onDraftChange: (draft: ProjectDraft) => void;
  onClose: () => void;
  onSubmit: (event: React.FormEvent<HTMLFormElement>) => void;
}

export const CreateProjectDialog = ({
  open,
  organizations,
  draft,
  isSubmitting,
  licenseTemplates,
  gitignoreTemplates,
  isLoadingLicenses,
  isLoadingGitignores,
  licenseError,
  gitignoreError,
  onRetryLicenses,
  onRetryGitignores,
  onDraftChange,
  onClose,
  onSubmit,
}: CreateProjectDialogProps): JSX.Element => {
  const { t } = useI18n();
  const update = <Key extends keyof ProjectDraft>(key: Key, value: ProjectDraft[Key]) => {
    onDraftChange({ ...draft, [key]: value });
  };

  return (
    <FormDialog open={open} onClose={onClose} title={t("Create Project")}>
      <form className="grid gap-3 md:grid-cols-2" onSubmit={onSubmit}>
        <div className="flex flex-col gap-2 md:col-span-2">
          <Label htmlFor="repo-owner">{t("Owner")}</Label>
          <Select value={draft.ownerOrganizationId} onValueChange={(value) => update("ownerOrganizationId", value)} required>
            <SelectTrigger id="repo-owner"><SelectValue placeholder={t("Owner")} /></SelectTrigger>
            <SelectContent>
              {organizations.map((organization) => (
                <SelectItem key={organization.id} value={organization.id}>{organization.name}</SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor="repo-key">{t("Project key")}</Label>
          <Input id="repo-key" placeholder={t("Project key")} value={draft.key} onChange={(event) => update("key", event.target.value)} required />
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor="repo-name">{t("Project name")}</Label>
          <Input id="repo-name" placeholder={t("Project Name")} value={draft.name} onChange={(event) => update("name", event.target.value)} required />
        </div>
        <div className="flex flex-col gap-2 md:col-span-2">
          <Label htmlFor="repo-description">{t("Description")}</Label>
          <Input id="repo-description" placeholder={t("Description (optional)")} value={draft.description} onChange={(event) => update("description", event.target.value)} />
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor="repo-default-branch">{t("Default branch")}</Label>
          <Input id="repo-default-branch" placeholder={t("Default branch")} value={draft.defaultBranch} onChange={(event) => update("defaultBranch", event.target.value)} />
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor="repo-visibility">{t("Visibility")}</Label>
          <Select value={draft.visibility} onValueChange={(value) => update("visibility", value)}>
            <SelectTrigger id="repo-visibility"><SelectValue placeholder={t("Visibility")} /></SelectTrigger>
            <SelectContent>
              <SelectItem value="private">{t("private")}</SelectItem>
              <SelectItem value="internal">{t("internal")}</SelectItem>
              <SelectItem value="public">{t("public")}</SelectItem>
            </SelectContent>
          </Select>
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor="repo-gitignore">{t("Add .gitignore")}</Label>
          <Select value={draft.gitignoreTemplate} onValueChange={(value) => update("gitignoreTemplate", value)}>
            <SelectTrigger id="repo-gitignore"><SelectValue placeholder={t("Add .gitignore")} /></SelectTrigger>
            <SelectContent>
              <SelectItem value="none">{t("None")}</SelectItem>
              {gitignoreTemplates.map((template) => (
                <SelectItem key={template.id} value={template.id}>{template.name}</SelectItem>
              ))}
            </SelectContent>
          </Select>
          {isLoadingGitignores ? <p className="text-xs text-muted-foreground">{t("Loading .gitignore templates...")}</p> : null}
          {gitignoreError ? (
            <div className="flex items-center justify-between gap-2 text-xs text-destructive">
              <span>{t("Could not load .gitignore templates.")}</span>
              <Button type="button" size="sm" variant="outline" onClick={onRetryGitignores}>{t("Retry")}</Button>
            </div>
          ) : null}
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor="repo-license">{t("Add license")}</Label>
          <Select value={draft.licenseTemplate} onValueChange={(value) => update("licenseTemplate", value)}>
            <SelectTrigger id="repo-license"><SelectValue placeholder={t("Add license")} /></SelectTrigger>
            <SelectContent>
              <SelectItem value="none">{t("None")}</SelectItem>
              {licenseTemplates.map((license) => (
                <SelectItem key={license.id} value={license.id}>{license.name}</SelectItem>
              ))}
            </SelectContent>
          </Select>
          {isLoadingLicenses ? <p className="text-xs text-muted-foreground">{t("Loading license templates...")}</p> : null}
          {licenseError ? (
            <div className="flex items-center justify-between gap-2 text-xs text-destructive">
              <span>{t("Could not load license templates.")}</span>
              <Button type="button" size="sm" variant="outline" onClick={onRetryLicenses}>{t("Retry")}</Button>
            </div>
          ) : null}
        </div>
        {isCopyrightOverrideSupported(draft.licenseTemplate) ? (
          <div className="flex flex-col gap-2">
            <Label htmlFor="repo-license-copyright-holder">{t("Copyright holder")}</Label>
            <Input
              id="repo-license-copyright-holder"
              placeholder={t("Defaults to the selected organization name.")}
              value={draft.licenseCopyrightHolder}
              onChange={(event) => update("licenseCopyrightHolder", event.target.value)}
            />
          </div>
        ) : null}
        <div className="flex items-start gap-2 md:col-span-2">
          <Checkbox
            id="repo-initialize-readme"
            checked={draft.initializeWithReadme}
            onCheckedChange={(checked) => update("initializeWithReadme", checked === true)}
          />
          <div className="flex flex-col gap-1">
            <Label htmlFor="repo-initialize-readme">{t("Initialize with a README")}</Label>
            <p className="text-xs text-muted-foreground">{t("Create the first commit with a README file.")}</p>
          </div>
        </div>
        <div className="flex items-center justify-end gap-2 border-t border-border/70 pt-3 md:col-span-2">
          <Button type="button" variant="outline" onClick={onClose}>{t("Cancel")}</Button>
          <Button type="submit" disabled={isSubmitting}>{isSubmitting ? t("Creating...") : t("Create")}</Button>
        </div>
      </form>
    </FormDialog>
  );
};

const isCopyrightOverrideSupported = (templateId: string): boolean =>
  templateId === "mit" || templateId === "bsd-2-clause" || templateId === "bsd-3-clause";
