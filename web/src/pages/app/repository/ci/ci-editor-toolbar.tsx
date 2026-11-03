import { ArrowLeft, FileCode2, Upload } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
interface Props {
  branch: string;
  branches: string[];
  dirty: boolean;
  mode: "canvas" | "yaml";
  canValidate: boolean;
  canCommit: boolean;
  onMode: (mode: "canvas" | "yaml") => void;
  onBranch: (branch: string) => void;
  onBack: () => void;
  onValidate: () => void;
  onCommit: () => void;
  t: (key: string) => string;
}
export const CIEditorToolbar = ({
  branch,
  branches,
  dirty,
  mode,
  canValidate,
  canCommit,
  onMode,
  onBranch,
  onBack,
  onValidate,
  onCommit,
  t,
}: Props) => <header className="flex flex-wrap items-center justify-between gap-3">
      <div className="flex flex-wrap items-center gap-3">
        <Button size="icon" variant="ghost" aria-label={t("Back to pipelines")} onClick={onBack}>
          <ArrowLeft className="size-4" />
        </Button>
        <h2 className="text-2xl font-semibold tracking-tight">{t("Edit pipeline")}</h2>
        <Badge variant="outline">
          <FileCode2 className="mr-1 size-3" />
          .gity-ci.yml
        </Badge>
        <select
          aria-label={t("Branch")}
          className="h-9 rounded-md border border-input bg-card px-2 text-sm"
          value={branch}
          onChange={(event) => onBranch(event.target.value)}
        >
          {branches.map((name) => (
            <option key={name}>{name}</option>
          ))}
        </select>
        <Badge variant="secondary">{t(dirty ? "Unsaved changes" : "Saved")}</Badge>
      </div>
      <div className="flex flex-wrap gap-2">
        <Button variant={mode === "canvas" ? "default" : "outline"} onClick={() => onMode("canvas")}>
          {t("Canvas")}
        </Button>
        <Button variant={mode === "yaml" ? "default" : "outline"} onClick={() => onMode("yaml")}>
          YAML
        </Button>
        <Button variant="outline" disabled={!canValidate} onClick={onValidate}>
          {t("Validate configuration")}
        </Button>
        <Button disabled={!canCommit} onClick={onCommit}>
          <Upload className="mr-2 size-4" />
          {t("Commit configuration")}
        </Button>
      </div>
    </header>;
