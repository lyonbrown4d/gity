import { Button } from "@/components/ui/button";
import { FormDialog } from "@/components/common/form-dialog";
interface Props {
  open: boolean;
  onKeep: () => void;
  onDiscard: () => void;
  t: (key: string) => string;
}
export const EditorLeaveDialog = ({ open, onKeep, onDiscard, t }: Props) => <FormDialog open={open} title={t("Unsaved changes")} onClose={onKeep}>
      <p className="mb-4 text-sm">{t("Keep editing or discard the unsaved configuration.")}</p>
      <div className="flex justify-end gap-2">
        <Button variant="outline" onClick={onKeep}>
          {t("Keep editing")}
        </Button>
        <Button variant="destructive" onClick={onDiscard}>
          {t("Discard changes")}
        </Button>
      </div>
    </FormDialog>;
