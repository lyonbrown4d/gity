import { LoaderCircle, Plus } from "lucide-react";
import { FormDialog } from "@/components/common/form-dialog";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Field, FieldError, FieldLabel } from "@/components/ui/field";
import { FileUpload } from "@/components/common/file-upload";
import { SelectControl } from "@/components/common/select-control";
import { formatBytes } from "./file-transfer";
import { usePackageUpload } from "./use-package-upload";

interface Props {
  repoId: string;
  canUpload: boolean;
  t: (key: string) => string;
  onUploaded: (identity: { name: string; type: string }) => Promise<void>;
}

export const PackageUploadDialog = ({ repoId, canUpload, t, onUploaded }: Props) => {
  const upload = usePackageUpload(repoId, t, onUploaded);
  const { values, form, pending, reading } = upload;
  const { errors } = form.formState;
  return (
    <FormDialog
      open={upload.open}
      onOpen={() => upload.setOpen(true)}
      onClose={upload.close}
      busy={pending}
      title={t("Upload package file")}
      description={t("Choose a file. Package details are filled in automatically and can be adjusted.")}
      trigger={
        <Button type="button" size="sm" disabled={!canUpload}>
          <Plus data-icon="inline-start" />
          {t("Upload package file")}
        </Button>
      }
    >
      <form
        noValidate
        className="space-y-5"
        onSubmit={(event) => {
          if (canUpload) void upload.submit(event);
          else event.preventDefault();
        }}
      >
        <fieldset disabled={pending} className="space-y-4">
          <FileUpload
            label={t("Package file")}
            name={upload.selectedFileName}
            metadata={`${formatBytes(upload.fileSize)} · ${values.content_type}`}
            disabled={pending || reading}
            reading={reading}
            t={t}
            onSelect={(file) => void upload.selectFile(file)}
            onReject={upload.setError}
          />
          <div className="grid gap-4 sm:grid-cols-[minmax(0,1fr)_160px]">
            <Field data-invalid={Boolean(errors.name)}>
              <FieldLabel htmlFor="package-name">{t("Package name")}</FieldLabel>
              <Input
                id="package-name"
                {...form.register("name")}
                aria-invalid={Boolean(errors.name)}
                aria-describedby={errors.name ? "package-name-error" : undefined}
                placeholder={t("Filled from the file name")}
              />
              {errors.name && <FieldError id="package-name-error" errors={[errors.name]} />}
            </Field>
            <Field data-invalid={Boolean(errors.version)}>
              <FieldLabel htmlFor="package-version">{t("Version")}</FieldLabel>
              <Input
                id="package-version"
                {...form.register("version")}
                aria-invalid={Boolean(errors.version)}
                aria-describedby={errors.version ? "package-version-error" : undefined}
              />
              {errors.version && <FieldError id="package-version-error" errors={[errors.version]} />}
            </Field>
          </div>
          <details className="rounded-md border border-border p-3">
            <summary className="cursor-pointer text-sm font-medium">{t("Advanced options")}</summary>
            <div className="mt-4 space-y-4">
              <div className="space-y-2">
                <FieldLabel>{t("Package type")}</FieldLabel>
                <SelectControl
                  label={t("Package type")}
                  value={values.type}
                  onChange={(value) => form.setValue("type", value as typeof values.type, { shouldDirty: true })}
                  options={["generic", "npm", "maven", "pypi", "nuget"].map((type) => ({ value: type, label: type }))}
                />
              </div>
              <div className="space-y-2">
                <FieldLabel htmlFor="package-file-path">{t("File path optional")}</FieldLabel>
                <Input
                  id="package-file-path"
                  {...form.register("file_path")}
                  placeholder={upload.selectedFileName || t("Same as the file name")}
                />
              </div>
            </div>
          </details>
        </fieldset>
        {upload.error && (
          <Alert variant="destructive">
            <AlertDescription>{upload.error}</AlertDescription>
          </Alert>
        )}
        <div className="flex flex-wrap justify-end gap-2 border-t border-border pt-4">
          <Button
            type="button"
            variant="outline"
            className="min-h-11 sm:min-h-9"
            disabled={pending}
            onClick={upload.close}
          >
            {t("Cancel")}
          </Button>
          <Button
            type="submit"
            className="min-h-11 sm:min-h-9"
            disabled={!canUpload || pending || reading || !upload.selectedFileName}
          >
            {pending && <LoaderCircle className="size-4 animate-spin motion-reduce:animate-none" aria-hidden="true" />}
            {pending ? t("Uploading package file...") : t("Upload file")}
          </Button>
        </div>
      </form>
    </FormDialog>
  );
};
