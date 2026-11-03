import { useEffect, useState } from "react";
import { useCustom } from "@refinedev/core";
import type { CiJob } from "@/lib/generated/model/ciJob";
import type { CiJobTrace } from "@/lib/generated/model/ciJobTrace";
import { isTerminal } from "./status";
import { tracePageSize } from "./trace-limits";
export { tracePageSize } from "./trace-limits";
export const useJobTrace = (repoId: string, job: CiJob, polling: boolean) => {
  const [attempt, setAttempt] = useState<number | null>(null);
  const [offset, setOffset] = useState(0);
  const [following, setFollowing] = useState(job.status !== "failed");
  const [lastReceived, setLastReceived] = useState<CiJobTrace | undefined>();
  const trace = useCustom<CiJobTrace>({
    url: `/projects/${repoId}/jobs/${job.id}/trace`,
    method: "get",
    config: { query: { limit: tracePageSize, offset, ...(attempt === null ? {} : { attempt }) } },
    queryOptions: {
      refetchOnWindowFocus: false,
      // Old segments must not accumulate in the request cache while browsing long logs.
      gcTime: 0,
      refetchInterval: (query) =>
        polling && !isTerminal(query.state.data?.data?.job?.status ?? job.status) ? 3000 : false,
    },
  });
  useEffect(() => {
    if (trace.result.data) setLastReceived(trace.result.data);
  }, [trace.result.data]);
  const data = trace.result.data ?? lastReceived;
  useEffect(() => {
    if (!following || trace.query.isFetching || !trace.result.data) return;
    const tail = Math.max(0, (trace.result.data.total_bytes ?? 0) - tracePageSize);
    if (tail !== offset) setOffset(tail);
  }, [following, offset, trace.query.isFetching, trace.result.data]);
  const selectAttempt = (value: number | null) => {
    setAttempt(value);
    setOffset(0);
    setFollowing(false);
    setLastReceived(undefined);
  };
  const move = (value: number) => {
    setFollowing(false);
    setOffset(Math.max(0, value));
  };
  return {
    data,
    query: trace.query,
    attempt,
    following,
    setFollowing,
    selectAttempt,
    beginning: () => move(0),
    previous: () => move((data?.offset ?? offset) - tracePageSize),
    next: () => move(data?.next_offset ?? offset),
    latest: () => setFollowing(true),
  };
};
