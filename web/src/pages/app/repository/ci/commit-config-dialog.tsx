import { lazy, Suspense, useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { FormDialog } from "@/components/common/form-dialog";
import { useTheme } from "@/lib/theme";
import { registerEditorTheme } from "@/lib/monaco-theme";
const DiffEditor = lazy(() => import("@/components/code-editor").then((module) => ({ default: module.DiffEditor })));
interface Props {
  open: boolean;
  onClose: () => void;
  branch: string;
  head: string;
  original: string;
  source: string;
  pending: boolean;
  error: string | null;
  onSave: (message: string) => Promise<boolean>;
  t: (key: string) => string;
}
export const CommitConfigDialog = ({
  open,
  onClose,
  branch,
  head,
  original,
  source,
  pending,
  error,
  onSave,
  t,
}: Props) => {
  const { editorTheme } = useTheme();
  const [message, setMessage] = useState("Update .gity-ci.yml");
  return (
    <FormDialog
      open={open}
      title={t("Commit configuration")}
      onClose={() => {
        if (!pending) onClose();
      }}
    >
      <form
        className="space-y-4"
        onSubmit={(event) => {
          event.preventDefault();
          if (!pending)
            void onSave(message).then((success) => {
              if (success) onClose();
            });
        }}
      >
        <p className="text-sm">
          {branch} · {head.slice(0, 12)} · .gity-ci.yml
        </p>
        <Suspense fallback={<p>{t("Loading editor...")}</p>}>
          <DiffEditor
            height="280px"
            language="yaml"
            original={original}
            modified={source}
            beforeMount={registerEditorTheme}
            theme={editorTheme}
            options={{ readOnly: true, renderSideBySide: false, minimap: { enabled: false } }}
          />
        </Suspense>
        <label className="block space-y-2 text-sm">
          {t("Commit message")}
          <Input value={message} onChange={(event) => setMessage(event.target.value)} required />
        </label>
        {error && (
          <p role="alert" className="text-sm text-destructive">
            {error}
          </p>
        )}
        <Button type="submit" disabled={pending || !message.trim()}>
          {t(pending ? "Committing..." : "Commit configuration")}
        </Button>
        <p className="text-xs text-muted-foreground">
          {t("Saving does not run a pipeline. Run the committed revision from pipelines.")}
        </p>
      </form>
    </FormDialog>
  );
};
