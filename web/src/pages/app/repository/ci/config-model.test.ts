import { describe, expect, it } from "vitest";
import { connectJobs, editJob, needsOf, parseConfig } from "./config-model";

const source = `# project CI\nname: Example\nstages: [build, test, deploy]\nvariables:\n  GREETING: hello\njobs:\n  build:\n    stage: build\n    script: cargo build\n  test:\n    stage: test\n    script: [cargo test]\n  independent:\n    stage: deploy\n    needs: []\n    script: echo ready\n`;
const parsed = (text: string) => {
  const result = parseConfig(text);
  if (!result.config) throw new Error(result.errors.join("\n"));
  return result.config;
};
describe("CI YAML execution semantics", () => {
  it("keeps absent dependencies distinct from an explicitly empty list after editing", () => {
    const before = parsed(source);
    const result = editJob(source, "test", { ...before.jobs.test, image: "rust:latest" });
    const after = parsed(result);
    expect(after.jobs.test).not.toHaveProperty("needs");
    expect(after.jobs.independent.needs).toEqual([]);
    expect(needsOf(after, "test")).toEqual(["build"]);
    expect(needsOf(after, "independent")).toEqual([]);
    expect(result).toContain("# project CI");
    expect(after.variables).toEqual({ GREETING: "hello" });
  });
  it("renames explicit references and removes only references to the deleted job", () => {
    const explicit = source.replace("script: [cargo test]", "script: [cargo test]\n    needs: [build]");
    const renamed = editJob(explicit, "build", parsed(explicit).jobs.build, "compile");
    expect(parsed(renamed).jobs.test.needs).toEqual(["compile"]);
    expect(parsed(editJob(renamed, "compile", null)).jobs.test.needs).toEqual([]);
  });
  it("blocks unsupported fields and malformed YAML without rewriting the source", () => {
    expect(parseConfig(source + "unsupported: true\n").config).toBeNull();
    expect(
      parseConfig(source.replace("script: cargo build", "script: cargo build\n    allow_failure: true")).errors.join(),
    ).toContain("allow_failure");
    const invalid = "jobs: [\n";
    expect(parseConfig(invalid).errors.join()).toContain("line");
    expect(() => editJob(invalid, "build", null)).toThrow();
    expect(invalid).toBe("jobs: [\n");
  });
  it("validates same-stage dependencies and rejects duplicate, self, forward and cyclic edges", () => {
    const sameStage = "stages: [test]\njobs:\n  a: {script: echo a, needs: []}\n  b: {script: echo b, needs: []}\n";
    const connected = connectJobs(sameStage, "a", "b");
    expect(parsed(connected).jobs.b.needs).toEqual(["a"]);
    expect(() => connectJobs(connected, "a", "b")).toThrow("Duplicate");
    expect(() => connectJobs(connected, "b", "a")).toThrow(/Cyclic/);
    expect(() => connectJobs(sameStage, "a", "a")).toThrow(/dependency/);
    expect(() => connectJobs(source, "independent", "build")).toThrow(/earlier stage/);
  });
  it("rejects zero or more than 200 jobs and invalid executable fields", () => {
    expect(parseConfig("jobs: {}\n").config).toBeNull();
    const many = `jobs:\n${Array.from({ length: 201 }, (_, i) => `  job-${i}: {script: echo ok}`).join("\n")}`;
    expect(parseConfig(many).config).toBeNull();
    for (const job of [
      "{script: ''}",
      "{script: echo ok, timeout_seconds: 0}",
      "{script: echo ok, artifacts: ['../secret']}",
      "{script: echo ok, variables: {lowercase: value}}",
    ])
      expect(parseConfig(`jobs: {task: ${job}}`).config).toBeNull();
    const last = "jobs:\n  one: {script: echo one}\n";
    expect(() => editJob(last, "one", null)).toThrow(/1–200/);
  });
});
