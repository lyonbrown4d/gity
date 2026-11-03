import { describe, expect, it, vi } from "vitest";
import { writeJobLog, type BoundedLogSlice } from "./log-download";
const writer = () => ({
    write: vi.fn(async (_bytes: Uint8Array<ArrayBuffer>) => {}),
    close: vi.fn(async () => {}),
    abort: vi.fn(async () => {}),
  });
describe("Bounded complete log download", () => {
  it("streams a fixed attempt and snapshot with Unicode boundaries while new output is appended", async () => {
    const load = vi.fn<(offset: number, attempt?: number) => Promise<BoundedLogSlice>>();
    load.mockResolvedValueOnce({ attempt: 2, offset: 0, next_offset: 4, total_bytes: 6, output: "A中" });
    load.mockResolvedValueOnce({ attempt: 2, offset: 4, next_offset: 11, total_bytes: 11, output: "BCextra" });
    const output = writer();
    await writeJobLog(load, output, { signal: new AbortController().signal });
    expect(load.mock.calls).toEqual([
      [0, undefined],
      [4, 2],
    ]);
    expect(output.write.mock.calls.map(([bytes]) => new TextDecoder().decode(bytes)).join("")).toBe("A中BC");
    expect(output.close).toHaveBeenCalledOnce();
    expect(output.abort).not.toHaveBeenCalled();
  });
  it("aborts the destination after a failed segment instead of publishing partial logs", async () => {
    const load = vi.fn<(offset: number, attempt?: number) => Promise<BoundedLogSlice>>();
    load.mockResolvedValueOnce({ attempt: 1, offset: 0, next_offset: 1, total_bytes: 2, output: "a" });
    load.mockRejectedValueOnce(new Error("offline"));
    const output = writer();
    await expect(writeJobLog(load, output, { signal: new AbortController().signal })).rejects.toThrow("offline");
    expect(output.abort).toHaveBeenCalledOnce();
    expect(output.close).not.toHaveBeenCalled();
  });
  it("rejects non-progressing pages and honors cancellation without a request loop", async () => {
    const output = writer();
    const load = vi.fn(async (): Promise<BoundedLogSlice> => ({
      attempt: 1,
      offset: 0,
      next_offset: 0,
      total_bytes: 1,
      output: "",
    }));
    await expect(writeJobLog(load, output, { signal: new AbortController().signal })).rejects.toThrow("log changed");
    expect(load).toHaveBeenCalledOnce();
    const cancel = new AbortController();
    cancel.abort();
    load.mockClear();
    await expect(writeJobLog(load, writer(), { signal: cancel.signal })).rejects.toThrow();
    expect(load).not.toHaveBeenCalled();
  });
});
