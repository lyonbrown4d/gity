import { parseDocument, type Document } from "yaml";
import { z } from "zod";

const variables = z.record(z.string().regex(/^[A-Z_][A-Z0-9_]{0,127}$/), z.string());
const strings = z.array(z.string());
const script = z.union([z.string(), strings]).refine((value) => {
  const lines = typeof value === "string" ? [value] : value;
  return lines.length > 0 && lines.every((line) => Boolean(line.trim()) && !line.includes("\0"));
}, "Each job requires a nonempty script");
const path = z
  .string()
  .refine(
    (value) =>
      Boolean(value) &&
      !/[\\\0*?[]/.test(value) &&
      value.split("/").every((part) => Boolean(part) && part !== "." && part !== ".." && part.toLowerCase() !== ".git"),
    "Artifact paths must be safe relative paths",
  );
const job = z.strictObject({
  stage: z.string().optional(),
  script,
  image: z
    .string()
    .refine((v) => !v.startsWith("-") && !v.includes("\0"))
    .optional(),
  tags: strings.optional(),
  needs: strings.nullable().optional(),
  variables: variables.optional(),
  artifacts: z.union([z.array(path), z.strictObject({ paths: z.array(path) })]).optional(),
  timeout_seconds: z.number().int().min(1).max(86400).optional(),
});
const config = z.strictObject({
  name: z.string().optional(),
  stages: strings.optional(),
  image: z.string().optional(),
  variables: variables.optional(),
  jobs: z
    .record(z.string().min(1).max(128), job)
    .refine((v) => Object.keys(v).length >= 1 && Object.keys(v).length <= 200, "CI configuration requires 1–200 jobs"),
});
export type CIConfig = z.infer<typeof config>;
export type CIJob = z.infer<typeof job>;
export type ParsedConfig =
  { config: CIConfig; document: Document; errors: [] } | { config: null; document: Document; errors: string[] };
export const defaultConfig =
  "name: CI\nstages: [build, test, deploy]\njobs:\n  test:\n    stage: test\n    script:\n      - cargo test --workspace\n";
export const stagesOf = (value: CIConfig): string[] =>
  value.stages?.length ? value.stages : ["build", "test", "deploy"];
export const needsOf = (value: CIConfig, name: string): string[] => {
  const item = value.jobs[name];
  if (item.needs != null) return item.needs;
  const stages = stagesOf(value);
  return Object.keys(value.jobs).filter(
    (other) => stages.indexOf(value.jobs[other].stage ?? "test") < stages.indexOf(item.stage ?? "test"),
  );
};
export const validateDependencies = (value: CIConfig): string[] => {
  const errors: string[] = [];
  const stages = stagesOf(value);
  if (new Set(stages).size !== stages.length || stages.some((stage) => !stage))
    errors.push("CI stages must be nonempty and unique");
  for (const [name, item] of Object.entries(value.jobs)) {
    const stage = stages.indexOf(item.stage ?? "test");
    if (stage < 0) errors.push(`${name}: unknown stage`);
    const needs = needsOf(value, name);
    if (new Set(needs).size !== needs.length) errors.push(`${name}: duplicate dependency`);
    for (const need of needs) {
      if (!value.jobs[need]) errors.push(`${name}: unknown dependency ${need}`);
      else if (need === name || stages.indexOf(value.jobs[need].stage ?? "test") > stage)
        errors.push(`${name}: dependency must be in the same or an earlier stage`);
    }
  }
  const done = new Set<string>();
  for (let pass = 0; pass < Object.keys(value.jobs).length; pass += 1) {
    for (const name of Object.keys(value.jobs))
      if (needsOf(value, name).every((need) => done.has(need))) done.add(name);
  }
  if (done.size !== Object.keys(value.jobs).length) errors.push("Cyclic CI job dependencies");
  return errors;
};
export const parseConfig = (source: string): ParsedConfig => {
  const document = parseDocument(source, { uniqueKeys: true });
  if (new TextEncoder().encode(source).length > 1024 * 1024)
    return { config: null, document, errors: ["CI configuration exceeds 1 MiB"] };
  if (document.errors.length) return { config: null, document, errors: document.errors.map((error) => error.message) };
  try {
    const parsed = config.safeParse(document.toJS({ maxAliasCount: 100 }));
    if (!parsed.success)
      return {
        config: null,
        document,
        errors: parsed.error.issues.map((issue) => `${issue.path.join(".")}: ${issue.message}`),
      };
    const errors = validateDependencies(parsed.data);
    return errors.length ? { config: null, document, errors } : { config: parsed.data, document, errors: [] };
  } catch (error) {
    return { config: null, document, errors: [error instanceof Error ? error.message : String(error)] };
  }
};
/** Edits the YAML document, preserving unrelated fields and absent dependency mode. */
export const editJob = (source: string, name: string, next: CIJob | null, nextName = name): string => {
  const parsed = parseConfig(source);
  if (!parsed.config) throw new Error(parsed.errors.join("\n"));
  const doc = parsed.document;
  if (nextName !== name && parsed.config.jobs[nextName]) throw new Error("Job name already exists");
  if (!next) {
    if (Object.keys(parsed.config.jobs).length <= 1) throw new Error("CI configuration requires 1–200 jobs");
    doc.deleteIn(["jobs", name]);
  } else {
    if (nextName !== name) doc.deleteIn(["jobs", name]);
    doc.setIn(["jobs", nextName], next);
  }
  for (const [other, item] of Object.entries(parsed.config.jobs)) {
    if (other === name || !item.needs?.includes(name)) continue;
    doc.setIn(
      ["jobs", other, "needs"],
      item.needs.flatMap((need) => (need === name ? (next ? [nextName] : []) : [need])),
    );
  }
  const result = doc.toString();
  const checked = parseConfig(result);
  if (!checked.config) throw new Error(checked.errors.join("\n"));
  return result;
};
export const connectJobs = (source: string, from: string, to: string): string => {
  const parsed = parseConfig(source);
  if (!parsed.config || !parsed.config.jobs[from] || !parsed.config.jobs[to]) throw new Error("Invalid dependency");
  const needs = needsOf(parsed.config, to);
  if (needs.includes(from)) throw new Error("Duplicate dependency");
  return editJob(source, to, { ...parsed.config.jobs[to], needs: [...needs, from] });
};
