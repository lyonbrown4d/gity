import { useState } from "react";
import { GitMerge } from "lucide-react";
import { Button } from "@/components/ui/button";
import { FormDialog } from "@/components/common/form-dialog";

interface Props {
  source: string;
  target: string;
  disabled: boolean;
  busy: boolean;
  reason: string | null;
  onMerge: () => void;
  t: (key: string) => string;
}

export const MergeConfirmation = ({ source, target, disabled, busy, reason, onMerge, t }: Props) => {
  const [open, setOpen] = useState(false);
  return (
    <FormDialog
      open={open}
      onOpen={() => setOpen(true)}
      onClose={() => setOpen(false)}
      busy={busy}
      title={t("Merge request")}
      description={t(
        "Merge the source branch into the target branch after the server rechecks permissions and merge conditions.",
      )}
      trigger={
        <Button
          type="button"
          size="sm"
          disabled={disabled}
          aria-describedby={reason ? "merge-disabled-reason" : undefined}
        >
          <GitMerge data-icon="inline-start" />
          {busy ? t("Merging...") : t("Merge")}
        </Button>
      }
    >
      <p className="rounded-md bg-muted p-3 text-sm">
        <code>{source}</code> → <code>{target}</code>
      </p>
      {reason && (
        <p role="alert" className="text-sm text-destructive">
          {reason}
        </p>
      )}
      <div className="flex justify-end gap-2">
        <Button type="button" variant="outline" disabled={busy} onClick={() => setOpen(false)}>
          {t("Cancel")}
        </Button>
        <Button
          type="button"
          disabled={disabled}
          onClick={() => {
            setOpen(false);
            onMerge();
          }}
        >
          {t("Merge")}
        </Button>
      </div>
    </FormDialog>
  );
};
