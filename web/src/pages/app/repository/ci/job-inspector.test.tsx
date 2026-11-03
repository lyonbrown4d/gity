// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { JobInspector } from "./job-inspector";
import { editJob, parseConfig } from "./config-model";
const source =
  "stages: [build, test]\njobs:\n  build: {stage: build, script: cargo build}\n  test: {stage: test, script: cargo test}\n";
const config = parseConfig(source).config!;
afterEach(cleanup);
describe("Task inspector", () => {
  it("exposes dependency modes and applies an explicit no-dependency job", () => {
    const apply = vi.fn();
    render(<JobInspector name="test" config={config} onApply={apply} onRemove={vi.fn()} t={(key) => key} />);
    fireEvent.change(screen.getByLabelText("Dependency mode"), { target: { value: "none" } });
    fireEvent.click(screen.getByRole("button", { name: "Apply task changes" }));
    expect(apply).toHaveBeenCalledOnce();
    expect(apply.mock.calls[0][1].needs).toEqual([]);
    expect(apply.mock.calls[0][1].script).toBe("cargo test");
  });
  it("preserves unrelated multiline scripts and literal tag and artifact values", () => {
    const job = { script: ["cd subdir\ncargo test"], tags: [" linux "], artifacts: [" report.txt"] };
    const apply = vi.fn();
    render(
      <JobInspector name="test" config={{ jobs: { test: job } }} onApply={apply} onRemove={vi.fn()} t={(key) => key} />,
    );
    fireEvent.change(screen.getByLabelText("Image"), { target: { value: "rust:latest" } });
    fireEvent.click(screen.getByRole("button", { name: "Apply task changes" }));
    const changed = apply.mock.calls[0][1];
    expect(changed.script).toEqual(job.script);
    expect(changed.tags).toEqual(job.tags);
    expect(changed.artifacts).toEqual(job.artifacts);
    expect(changed.image).toBe("rust:latest");
  });
  it("retains the form and reports validation failure without publishing an invalid YAML update", () => {
    const apply = vi.fn((name, job) => {
      editJob(source, "test", job, name);
    });
    render(<JobInspector name="test" config={config} onApply={apply} onRemove={vi.fn()} t={(key) => key} />);
    fireEvent.change(screen.getByLabelText("Task name"), { target: { value: "build" } });
    fireEvent.click(screen.getByRole("button", { name: "Apply task changes" }));
    expect(screen.getByRole("alert").textContent).toContain("already exists");
    expect((screen.getByLabelText("Task name") as HTMLInputElement).value).toBe("build");
    expect(parseConfig(source).config?.jobs.test.script).toBe("cargo test");
  });
});
