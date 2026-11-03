import { useId } from "react";
import { useDropzone } from "react-dropzone";
import { File, LoaderCircle, Upload } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Field, FieldDescription, FieldError, FieldLabel } from "@/components/ui/field";
import { cn } from "@/lib/utils";

interface Props {
  label: string;
  name: string;
  metadata: string;
  disabled: boolean;
  reading: boolean;
  onSelect: (file: File) => void;
  onReject: (message: string) => void;
  t: (key: string) => string;
}

/** File selection is managed by react-dropzone; shadcn supplies the visual controls. */
export const FileUpload = ({ label, name, metadata, disabled, reading, onSelect, onReject, t }: Props) => {
  const inputId = useId();
  const dropzone = useDropzone({
    multiple: false,
    maxFiles: 1,
    disabled,
    noClick: true,
    noKeyboard: true,
    onDropAccepted: (files) => {
      if (files[0]) onSelect(files[0]);
    },
    onDropRejected: () => onReject(t("Choose one file at a time.")),
  });
  return (
    <Field data-invalid={dropzone.isDragReject || undefined}>
      <FieldLabel htmlFor={inputId}>{label}</FieldLabel>
      <div
        {...dropzone.getRootProps()}
        className={cn(
          "space-y-3 rounded-md border border-dashed border-border bg-muted/20 p-5 transition-colors",
          dropzone.isDragActive && "border-primary bg-primary/5",
          disabled && "opacity-70",
        )}
      >
        <Input {...dropzone.getInputProps({ id: inputId, "aria-label": label })} />
        <div className="flex items-start gap-3">
          {reading ? (
            <LoaderCircle
              className="mt-1 size-6 shrink-0 animate-spin text-primary motion-reduce:animate-none"
              aria-hidden="true"
            />
          ) : name ? (
            <File className="mt-1 size-6 shrink-0 text-primary" aria-hidden="true" />
          ) : (
            <Upload className="mt-1 size-6 shrink-0 text-muted-foreground" aria-hidden="true" />
          )}
          <div className="min-w-0 flex-1">
            <p className="break-all text-sm font-medium">{name || t("Drop a file here")}</p>
            <p role="status" className="mt-1 break-all text-xs text-muted-foreground">
              {reading ? t("Reading file...") : name ? metadata : t("Or choose a file from your device.")}
            </p>
          </div>
        </div>
        <Button
          type="button"
          variant="outline"
          size="sm"
          className="min-h-11 sm:min-h-8"
          onClick={dropzone.open}
          disabled={disabled}
        >
          {t(name ? "Replace file" : "Choose file")}
        </Button>
        {dropzone.isDragReject && <FieldError>{t("Choose one file at a time.")}</FieldError>}
      </div>
      <FieldDescription>{t("File name, content type and default path come from the selected file.")}</FieldDescription>
    </Field>
  );
};
