import { useState } from "react";
import { Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { FormDialog } from "@/components/common/form-dialog";
import { needsOf, stagesOf, type CIConfig, type CIJob } from "./config-model";

interface Props {
  name: string;
  config: CIConfig;
  onApply: (name: string, job: CIJob) => void;
  onRemove: () => void;
  t: (key: string) => string;
}
const lines = (value: string) => value.split("\n").filter((line) => line.length > 0);
export const JobInspector = ({ name, config, onApply, onRemove, t }: Props) => {
  const job = config.jobs[name];
  const originalScript = typeof job.script === "string" ? job.script : job.script.join("\n");
  const originalArtifacts = (Array.isArray(job.artifacts) ? job.artifacts : (job.artifacts?.paths ?? [])).join("\n");
  const [nextName, setName] = useState(name);
  const [stage, setStage] = useState(job.stage ?? "test");
  const [image, setImage] = useState(job.image ?? "");
  const [scriptMode, setScriptMode] = useState(typeof job.script === "string" ? "single" : "list");
  const [script, setScript] = useState(originalScript);
  const [mode, setMode] = useState(job.needs == null ? "stage" : job.needs.length ? "explicit" : "none");
  const [needs, setNeeds] = useState(job.needs ?? []);
  const [timeout, setTimeoutValue] = useState(String(job.timeout_seconds ?? 3600));
  const [tags, setTags] = useState((job.tags ?? []).join("\n"));
  const [variables, setVariables] = useState(JSON.stringify(job.variables ?? {}, null, 2));
  const [artifacts, setArtifacts] = useState(originalArtifacts);
  const [error, setError] = useState<string | null>(null);
  const [confirmRemove, setConfirmRemove] = useState(false);
  const dependents = Object.keys(config.jobs).filter(
    (other) => other !== name && needsOf(config, other).includes(name),
  );
  const apply = (event: React.FormEvent) => {
    event.preventDefault();
    try {
      const next: CIJob = {
        ...job,
        stage: stage === (job.stage ?? "test") ? job.stage : stage,
        image: image === (job.image ?? "") ? job.image : image,
        script:
          script === originalScript && scriptMode === (typeof job.script === "string" ? "single" : "list")
            ? job.script
            : scriptMode === "single"
              ? script
              : script.split("\n"),
        tags: tags === (job.tags ?? []).join("\n") ? job.tags : lines(tags),
        variables: variables === JSON.stringify(job.variables ?? {}, null, 2) ? job.variables : JSON.parse(variables),
        artifacts:
          artifacts === originalArtifacts
            ? job.artifacts
            : Array.isArray(job.artifacts) || !job.artifacts
              ? lines(artifacts)
              : { paths: lines(artifacts) },
        timeout_seconds: timeout === String(job.timeout_seconds ?? 3600) ? job.timeout_seconds : Number(timeout),
      };
      if (mode === "stage") delete next.needs;
      else next.needs = mode === "none" ? [] : needs;
      onApply(nextName, next);
      setError(null);
    } catch (failure) {
      setError(failure instanceof Error ? failure.message : String(failure));
    }
  };
  return (
    <aside className="w-full shrink-0 border-t border-border bg-card lg:max-h-[600px] lg:w-80 lg:overflow-auto lg:border-l lg:border-t-0">
      <form className="space-y-4 p-5" onSubmit={apply}>
        <h3 className="font-semibold">
          {t("Task configuration")} — {name}
        </h3>
        <Field label={t("Task name")}>
          <Input
            aria-label={t("Task name")}
            value={nextName}
            onChange={(event) => setName(event.target.value)}
            required
            maxLength={128}
          />
        </Field>
        <Field label={t("Stage")}>
          <select
            className="h-9 w-full rounded-md border border-input bg-background px-2 text-sm"
            aria-label={t("Stage")}
            value={stage}
            onChange={(event) => setStage(event.target.value)}
          >
            {stagesOf(config).map((item) => (
              <option key={item}>{item}</option>
            ))}
          </select>
        </Field>
        <Field label={t("Image")}>
          <Input
            aria-label={t("Image")}
            value={image}
            onChange={(event) => setImage(event.target.value)}
            placeholder={config.image || t("Default runner image")}
          />
        </Field>
        <Field label={t("Dependency mode")}>
          <select
            aria-label={t("Dependency mode")}
            className="h-9 w-full rounded-md border border-input bg-background px-2 text-sm"
            value={mode}
            onChange={(event) => setMode(event.target.value)}
          >
            <option value="stage">{t("Follow stages")}</option>
            <option value="explicit">{t("Explicit dependencies")}</option>
            <option value="none">{t("No dependencies")}</option>
          </select>
        </Field>
        {mode === "explicit" && (
          <fieldset className="max-h-36 space-y-2 overflow-auto rounded-md border border-border p-3">
            <legend className="text-xs">{t("Dependencies")}</legend>
            {Object.entries(config.jobs)
              .filter(
                ([other, item]) =>
                  other !== name && stagesOf(config).indexOf(item.stage ?? "test") <= stagesOf(config).indexOf(stage),
              )
              .map(([other]) => (
                <label key={other} className="flex items-center gap-2 text-sm">
                  <input
                    type="checkbox"
                    checked={needs.includes(other)}
                    onChange={(event) =>
                      setNeeds(event.target.checked ? [...needs, other] : needs.filter((need) => need !== other))
                    }
                  />
                  {other}
                </label>
              ))}
          </fieldset>
        )}
        <Field label={t("Script format")}>
          <select
            aria-label={t("Script format")}
            className="h-9 w-full rounded-md border border-input bg-background px-2 text-sm"
            value={scriptMode}
            onChange={(event) => setScriptMode(event.target.value)}
          >
            <option value="single">{t("Single script")}</option>
            <option value="list">{t("One command per line")}</option>
          </select>
        </Field>
        <Field label={t("Script")}>
          <Textarea
            aria-label={t("Script")}
            className="min-h-28 font-mono text-xs"
            value={script}
            onChange={(event) => setScript(event.target.value)}
            required
          />
        </Field>
        <Field label={t("Timeout (seconds)")}>
          <Input
            aria-label={t("Timeout (seconds)")}
            type="number"
            min={1}
            max={86400}
            value={timeout}
            onChange={(event) => setTimeoutValue(event.target.value)}
            required
          />
        </Field>
        <details>
          <summary className="cursor-pointer text-sm font-medium">{t("Environment variables")}</summary>
          <Textarea
            aria-label={t("Environment variables")}
            className="mt-2 font-mono text-xs"
            value={variables}
            onChange={(event) => setVariables(event.target.value)}
          />
          <p className="mt-1 text-xs text-muted-foreground">
            {t("JSON object; secrets belong in project CI variables.")}
          </p>
        </details>
        <details>
          <summary className="cursor-pointer text-sm font-medium">{t("Runner tags")}</summary>
          <Textarea
            aria-label={t("Runner tags")}
            className="mt-2"
            value={tags}
            onChange={(event) => setTags(event.target.value)}
          />
        </details>
        <details>
          <summary className="cursor-pointer text-sm font-medium">{t("Artifact paths")}</summary>
          <Textarea
            aria-label={t("Artifact paths")}
            className="mt-2"
            value={artifacts}
            onChange={(event) => setArtifacts(event.target.value)}
          />
        </details>
        {error && (
          <p role="alert" className="whitespace-pre-wrap text-xs text-destructive">
            {error}
          </p>
        )}
        <div className="flex gap-2">
          <Button type="submit" className="flex-1">
            {t("Apply task changes")}
          </Button>
          <Button
            type="button"
            variant="outline"
            size="icon"
            aria-label={t("Delete task")}
            disabled={Object.keys(config.jobs).length <= 1}
            onClick={() => setConfirmRemove(true)}
          >
            <Trash2 className="size-4" />
          </Button>
        </div>
      </form>
      <FormDialog open={confirmRemove} onClose={() => setConfirmRemove(false)} title={t("Delete task")}>
        <p className="text-sm">{t("Deleting this task removes its explicit dependency references.")}</p>
        <p className="my-4 text-sm text-muted-foreground">
          {t("Affected tasks")}: {dependents.join(", ") || t("None")}
        </p>
        <div className="flex justify-end gap-2">
          <Button variant="outline" onClick={() => setConfirmRemove(false)}>
            {t("Cancel")}
          </Button>
          <Button variant="destructive" onClick={onRemove}>
            {t("Delete task")}
          </Button>
        </div>
      </FormDialog>
    </aside>
  );
};
const Field = ({ label, children }: { label: string; children: React.ReactNode }) => <div className="space-y-2">
      <Label>{label}</Label>
      {children}
    </div>;
