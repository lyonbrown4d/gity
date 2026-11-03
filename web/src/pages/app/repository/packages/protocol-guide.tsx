import { useEffect, useState } from "react";
import { Copy, Terminal } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";

export interface PackageProtocolGuideItem {
  id: string;
  title: string;
  description: string;
  endpoint: string;
  scopes: string[];
  publishCommands: string[];
  installCommands: string[];
}

export const PackageProtocolGuide = ({
  guides,
  copiedCommand,
  t,
  onCopy,
}: {
  guides: PackageProtocolGuideItem[];
  copiedCommand: string | null;
  t: (text: string) => string;
  onCopy: (command: string) => void;
}) => {
  const [activeProtocol, setActiveProtocol] = useState("generic");
  const activeGuide = guides.find((guide) => guide.id === activeProtocol) ?? guides[0];

  useEffect(() => {
    if (!guides.some((guide) => guide.id === activeProtocol)) {
      setActiveProtocol(guides[0]?.id ?? "generic");
    }
  }, [activeProtocol, guides]);

  if (!activeGuide) {
    return null;
  }

  return (
    <div className="grid gap-3 border border-border bg-muted/20 p-3 lg:grid-cols-[240px_1fr]">
      <div className="flex flex-col gap-2">
        <div className="flex items-center gap-2">
          <Terminal className="size-4 text-primary" />
          <p className="font-medium">{t("Registry setup guide")}</p>
        </div>
        <p className="text-xs leading-5 text-muted-foreground">
          {t(
            "Copy publish and install commands for the selected package protocol. Use a project access token or deploy token with package scopes.",
          )}
        </p>
        <Select value={activeProtocol} onValueChange={setActiveProtocol}>
          <SelectTrigger aria-label={t("Package protocol")}>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              {guides.map((guide) => (
                <SelectItem key={guide.id} value={guide.id}>
                  {guide.title}
                </SelectItem>
              ))}
            </SelectGroup>
          </SelectContent>
        </Select>
        <div className="flex flex-wrap gap-1">
          {activeGuide.scopes.map((scope) => (
            <Badge key={scope} variant="outline">
              {scope}
            </Badge>
          ))}
        </div>
      </div>
      <div className="grid gap-3 xl:grid-cols-2">
        <div className="flex flex-col gap-2 border bg-background/80 p-3">
          <div>
            <p className="text-sm font-medium">{activeGuide.title}</p>
            <p className="text-xs text-muted-foreground">{t(activeGuide.description)}</p>
          </div>
          <PackageCommandLine
            label={t("Endpoint")}
            value={activeGuide.endpoint}
            copiedCommand={copiedCommand}
            t={t}
            onCopy={onCopy}
          />
          {activeGuide.publishCommands.map((command) => (
            <PackageCommandLine
              key={command}
              label={t("Publish")}
              value={command}
              copiedCommand={copiedCommand}
              t={t}
              onCopy={onCopy}
            />
          ))}
        </div>
        <div className="flex flex-col gap-2 border bg-background/80 p-3">
          <p className="text-sm font-medium">{t("Install or consume")}</p>
          {activeGuide.installCommands.map((command) => (
            <PackageCommandLine
              key={command}
              label={t("Install")}
              value={command}
              copiedCommand={copiedCommand}
              t={t}
              onCopy={onCopy}
            />
          ))}
        </div>
      </div>
    </div>
  );
};

const PackageCommandLine = ({
  label,
  value,
  copiedCommand,
  t,
  onCopy,
}: {
  label: string;
  value: string;
  copiedCommand: string | null;
  t: (text: string) => string;
  onCopy: (command: string) => void;
}) => (
  <div className="flex flex-col gap-1">
    <div className="flex items-center justify-between gap-2">
      <span className="text-xs font-medium text-muted-foreground">{label}</span>
      <Button type="button" size="sm" variant="ghost" className="h-7 px-2" onClick={() => onCopy(value)}>
        <Copy className="size-3.5" />
        {copiedCommand === value ? t("Copied") : t("Copy")}
      </Button>
    </div>
    <code className="block overflow-x-auto rounded-md bg-muted px-3 py-2 font-mono text-xs leading-5 text-foreground">
      {value}
    </code>
  </div>
);
