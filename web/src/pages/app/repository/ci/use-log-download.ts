import { useEffect, useRef, useState } from "react";
import { useDataProvider } from "@refinedev/core";
import type { CiJobTrace } from "@/lib/generated/model/ciJobTrace";
import { extractErrorMessage } from "../issues-utils";
import { writeJobLog } from "./log-download";
import { tracePageSize } from "./trace-limits";
type SavePicker = (options: {
  suggestedName: string;
  types: { description: string; accept: Record<string, string[]> }[];
}) => Promise<FileSystemFileHandle>;
export const useLogDownload = (repoId: string, jobId: string, attempt: number | null | undefined) => {
  const provider = useDataProvider();
  const controller = useRef<AbortController | null>(null);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [progress, setProgress] = useState<{ received: number; total: number } | null>(null);
  const picker =
    typeof window === "undefined"
      ? undefined
      : (window as Window & { showSaveFilePicker?: SavePicker }).showSaveFilePicker;
  useEffect(() => () => controller.current?.abort(), []);
  const start = async () => {
    if (!picker || controller.current) return;
    const active = new AbortController();
    controller.current = active;
    setPending(true);
    setError(null);
    setProgress(null);
    try {
      const handle = await picker.call(window, {
        suggestedName: `job-${jobId}${attempt != null ? `-attempt-${attempt}` : ""}.log`,
        types: [{ description: "Log", accept: { "text/plain": [".log"] } }],
      });
      const writer = await handle.createWritable();
      await writeJobLog(
        async (offset, selectedAttempt) => {
          const response = await provider().custom?.<CiJobTrace>({
            url: `/projects/${repoId}/jobs/${jobId}/trace`,
            method: "get",
            query: { offset, limit: tracePageSize, ...(selectedAttempt == null ? {} : { attempt: selectedAttempt }) },
            meta: { signal: active.signal },
          });
          if (!response) throw new Error("Log download is not available.");
          return response.data;
        },
        writer,
        {
          attempt: attempt ?? undefined,
          signal: active.signal,
          onProgress: (received, total) => setProgress({ received, total }),
        },
      );
    } catch (failure) {
      if (!(failure instanceof DOMException && failure.name === "AbortError") && !active.signal.aborted)
        setError(extractErrorMessage(failure));
    } finally {
      controller.current = null;
      setPending(false);
    }
  };
  return { supported: Boolean(picker), pending, progress, error, start, cancel: () => controller.current?.abort() };
};
