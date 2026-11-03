import { Button } from "@/components/ui/button";
import { FormDialog } from "@/components/common/form-dialog";
interface Props {
  open: boolean;
  head?: string;
  onRestore: () => void;
  onDiscard: () => void;
  t: (key: string) => string;
}
export const RecoverDraftDialog = ({ open, head, onRestore, onDiscard, t }: Props) => <FormDialog open={open} title={t("Recover unsaved configuration")} onClose={onDiscard}>
      <p className="text-sm">
        {t("A configuration draft was kept in this browser tab. Restore it or use the repository version.")}
      </p>
      <p className="my-4 font-mono text-xs text-muted-foreground">{head?.slice(0, 12)}</p>
      <div className="flex justify-end gap-2">
        <Button variant="outline" onClick={onDiscard}>
          {t("Use repository version")}
        </Button>
        <Button onClick={onRestore}>{t("Restore draft")}</Button>
      </div>
    </FormDialog>;
