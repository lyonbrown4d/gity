import Editor from "@monaco-editor/react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Modal } from "@/components/ui/modal";
import type { RepositoryBranchView } from "@/pages/types";
import { detectLanguage } from "./repository-utils";

interface RepositoryCreateFileModalProps {
  open: boolean;
  t: (text: string) => string;
  editorTheme: string;
  branches: RepositoryBranchView[];
  newFileBranch: string;
  newFilePath: string;
  newFileMessage: string;
  newFileContent: string;
  isCreatingFile: boolean;
  onClose: () => void;
  onChangeNewFileBranch: (value: string) => void;
  onChangeNewFilePath: (value: string) => void;
  onChangeNewFileMessage: (value: string) => void;
  onChangeNewFileContent: (value: string) => void;
  onSubmit: (event: React.FormEvent<HTMLFormElement>) => void;
}

export function RepositoryCreateFileModal({
  open,
  t,
  editorTheme,
  branches,
  newFileBranch,
  newFilePath,
  newFileMessage,
  newFileContent,
  isCreatingFile,
  onClose,
  onChangeNewFileBranch,
  onChangeNewFilePath,
  onChangeNewFileMessage,
  onChangeNewFileContent,
  onSubmit,
}: RepositoryCreateFileModalProps): JSX.Element {
  return (
    <Modal open={open} onClose={onClose} title={t("Create file and commit")}>
      <form className="space-y-3" onSubmit={onSubmit}>
        <div className="grid gap-3 md:grid-cols-2">
          <div className="space-y-2">
            <Label htmlFor="new-file-branch">{t("Branch")}</Label>
            <select
              id="new-file-branch"
              className="h-9 w-full rounded-md border bg-background px-3 text-sm"
              value={newFileBranch}
              onChange={(event) => onChangeNewFileBranch(event.target.value)}
              required
            >
              {branches.map((branch) => (
                <option key={branch.name} value={branch.name}>
                  {branch.name}
                </option>
              ))}
            </select>
          </div>
          <div className="space-y-2">
            <Label htmlFor="new-file-path">{t("File path")}</Label>
            <Input
              id="new-file-path"
              placeholder="src/new-file.ts"
              value={newFilePath}
              onChange={(event) => onChangeNewFilePath(event.target.value)}
              required
            />
          </div>
        </div>

        <div className="space-y-2">
          <Label htmlFor="new-file-message">{t("Commit message")}</Label>
          <Input
            id="new-file-message"
            placeholder={t("Commit message")}
            value={newFileMessage}
            onChange={(event) => onChangeNewFileMessage(event.target.value)}
            required
          />
        </div>

        <div className="space-y-2">
          <Label>{t("File content")}</Label>
          <Editor
            height="40vh"
            language={detectLanguage(newFilePath)}
            value={newFileContent}
            theme={editorTheme}
            onChange={(value) => onChangeNewFileContent(value ?? "")}
            options={{
              minimap: { enabled: false },
              fontSize: 13,
              wordWrap: "on",
              scrollBeyondLastLine: false,
            }}
          />
        </div>

        <div className="flex items-center justify-end gap-2">
          <Button type="button" variant="outline" onClick={onClose}>
            {t("Cancel")}
          </Button>
          <Button type="submit" disabled={isCreatingFile}>
            {isCreatingFile ? t("Committing...") : t("Commit and create file")}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
