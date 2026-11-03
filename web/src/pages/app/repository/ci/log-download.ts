import type { CiJobTrace } from "@/lib/generated/model/ciJobTrace";
import { tracePageSize } from "./trace-limits";
export type BoundedLogSlice = Pick<CiJobTrace, "attempt" | "total_bytes" | "offset" | "next_offset" | "output">;
export interface LogWriter {
  write: (bytes: Uint8Array<ArrayBuffer>) => Promise<void>;
  close: () => Promise<void>;
  abort: () => Promise<void>;
}
interface Options {
  attempt?: number;
  signal: AbortSignal;
  onProgress?: (received: number, total: number) => void;
}
/** Writes a fixed attempt snapshot in bounded chunks, including during live appends. */
export const writeJobLog = async (
  load: (offset: number, attempt?: number) => Promise<BoundedLogSlice>,
  writer: LogWriter,
  options: Options,
): Promise<void> => {
  try {
    options.signal.throwIfAborted();
    let page = await load(0, options.attempt);
    if (page.attempt == null || page.total_bytes == null)
      throw new Error("The server did not return a bounded log segment.");
    const attempt = page.attempt;
    const total = page.total_bytes;
    let offset = 0;
    while (offset < total) {
      options.signal.throwIfAborted();
      const bytes = new TextEncoder().encode(page.output);
      if (
        page.attempt !== attempt ||
        page.offset !== offset ||
        page.next_offset == null ||
        page.next_offset <= offset ||
        page.next_offset - offset !== bytes.length ||
        bytes.length > tracePageSize
      ) {
        throw new Error("The log changed while downloading. Retry the download.");
      }
      const remaining = total - offset;
      await writer.write(bytes.slice(0, remaining));
      offset = Math.min(page.next_offset, total);
      options.onProgress?.(offset, total);
      if (offset < total) page = await load(offset, attempt);
    }
    options.signal.throwIfAborted();
    await writer.close();
  } catch (error) {
    await writer.abort().catch(() => undefined);
    throw error;
  }
};
