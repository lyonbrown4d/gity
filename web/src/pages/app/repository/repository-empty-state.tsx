import { useState } from "react";
import { Check, Copy, Terminal } from "lucide-react";
import { Button } from "@/components/ui/button";

interface RepositoryEmptyStateProps {
  t: (text: string) => string;
  repositoryName: string;
  cloneHttpUrl: string;
  defaultBranch: string;
}

interface CommandSectionProps {
  title: string;
  commands: string[];
  copiedCommand: string | null;
  t: (text: string) => string;
  onCopy: (command: string) => void;
}

const CommandSection = ({
  title,
  commands,
  copiedCommand,
  t,
  onCopy,
}: CommandSectionProps): JSX.Element => (
  <div className="space-y-2 rounded-md border p-3">
    <p className="text-sm font-medium">{title}</p>
    <div className="space-y-2">
      {commands.map((command) => (
        <div key={command} className="flex items-center gap-2 rounded-md bg-muted/60 p-2">
          <code className="flex-1 overflow-x-auto whitespace-nowrap text-xs">{command}</code>
          <Button type="button" size="sm" variant="outline" onClick={() => onCopy(command)}>
            {copiedCommand === command ? (
              <>
                <Check className="mr-1 size-3" />
                {t("Copied")}
              </>
            ) : (
              <>
                <Copy className="mr-1 size-3" />
                {t("Copy")}
              </>
            )}
          </Button>
        </div>
      ))}
    </div>
  </div>
);

export const RepositoryEmptyState = ({
  t,
  repositoryName,
  cloneHttpUrl,
  defaultBranch,
}: RepositoryEmptyStateProps): JSX.Element => {
  const [copiedCommand, setCopiedCommand] = useState<string | null>(null);
  const bootstrapCommands = [
    "git init",
    `echo "# ${repositoryName}" > README.md`,
    "git add README.md",
    'git commit -m "first commit"',
    `git branch -M ${defaultBranch}`,
    `git remote add origin ${cloneHttpUrl}`,
    `git push -u origin ${defaultBranch}`,
  ];
  const pushExistingCommands = [
    `git remote add origin ${cloneHttpUrl}`,
    `git branch -M ${defaultBranch}`,
    `git push -u origin ${defaultBranch}`,
  ];

  const copyCommand = (command: string) => {
    void navigator.clipboard
      .writeText(command)
      .then(() => {
        setCopiedCommand(command);
        window.setTimeout(() => {
          setCopiedCommand((current) => (current === command ? null : current));
        }, 1400);
      })
      .catch(() => {
        // ignore clipboard errors in browser without permission
      });
  };

  return (
    <div className="space-y-4 p-4">
      <div className="rounded-md border border-dashed p-4">
        <div className="mb-2 flex items-center gap-2">
          <Terminal className="size-4" />
          <p className="text-sm font-semibold">{t("This repository is empty")}</p>
        </div>
        <p className="text-xs text-muted-foreground">{t("Push your code from the command line.")}</p>
      </div>
      <CommandSection
        title={t("Create a new repository on the command line")}
        commands={bootstrapCommands}
        copiedCommand={copiedCommand}
        t={t}
        onCopy={copyCommand}
      />
      <CommandSection
        title={t("Push an existing repository from the command line")}
        commands={pushExistingCommands}
        copiedCommand={copiedCommand}
        t={t}
        onCopy={copyCommand}
      />
    </div>
  );
};
