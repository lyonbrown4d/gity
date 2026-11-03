import { useEffect, useMemo, useRef, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { ArrowDown, ArrowUp, Copy, Download, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { saveBlob } from "./status";
import "./flow.scss";
interface Props {
  output: string;
  t: (key: string) => string;
  jobId: string;
  failed: boolean;
  following: boolean;
  onFollowingChange: (following: boolean) => void;
  onDownloadFull?: () => void;
  downloading?: boolean;
}
export const JobLogViewer = ({
  output,
  t,
  jobId,
  failed,
  following,
  onFollowingChange: setFollowing,
  onDownloadFull,
  downloading,
}: Props) => {
  const viewport = useRef<HTMLDivElement>(null);
  const [search, setSearch] = useState("");
  const [match, setMatch] = useState(0);
  const [copyMessage, setCopyMessage] = useState("");
  const lines = useMemo(() => output.split("\n"), [output]);
  const matches = useMemo(
    () =>
      search ? lines.flatMap((line, index) => (line.toLowerCase().includes(search.toLowerCase()) ? [index] : [])) : [],
    [lines, search],
  );
  const virtualizer = useVirtualizer({
    count: lines.length,
    getScrollElement: () => viewport.current,
    estimateSize: () => 22,
    overscan: 12,
  });
  const initialPositioned = useRef(false);
  useEffect(() => {
    if (!output) return;
    if (following) virtualizer.scrollToIndex(lines.length - 1, { align: "end" });
    else if (failed && !initialPositioned.current) {
      const firstFailure = lines.findIndex((line) => /error|failed|panic|fatal/i.test(line));
      if (firstFailure >= 0) virtualizer.scrollToIndex(firstFailure, { align: "center" });
    }
    initialPositioned.current = true;
  }, [output, lines, following, failed, virtualizer]);
  const jumpMatch = (direction: number) => {
    if (!matches.length) return;
    const next = (match + direction + matches.length) % matches.length;
    setMatch(next);
    setFollowing(false);
    virtualizer.scrollToIndex(matches[next], { align: "center" });
  };
  const copy = async () => {
    try {
      const selection = window.getSelection()?.toString();
      await navigator.clipboard.writeText(selection || output);
      setCopyMessage(t("Copied"));
    } catch {
      setCopyMessage(t("Copy failed"));
    }
  };
  return (
    <div className="min-w-0 overflow-hidden rounded-lg border border-border bg-card">
      <div className="flex flex-wrap items-center gap-2 border-b border-border p-3">
        <Input
          className="max-w-64"
          aria-label={t("Search loaded segment")}
          placeholder={t("Search loaded segment")}
          value={search}
          onChange={(event) => {
            const query = event.target.value;
            setSearch(query);
            setMatch(0);
            const first = lines.findIndex((line) => line.toLowerCase().includes(query.toLowerCase()));
            if (query && first >= 0) {
              setFollowing(false);
              virtualizer.scrollToIndex(first, { align: "center" });
            }
          }}
        />
        <span className="text-xs text-muted-foreground">
          {matches.length ? `${match + 1} / ${matches.length}` : search ? t("No matches") : ""}
        </span>
        <Button
          size="icon"
          variant="outline"
          aria-label={t("Previous match")}
          disabled={!matches.length}
          onClick={() => jumpMatch(-1)}
        >
          <ArrowUp className="size-4" />
        </Button>
        <Button
          size="icon"
          variant="outline"
          aria-label={t("Next match")}
          disabled={!matches.length}
          onClick={() => jumpMatch(1)}
        >
          <ArrowDown className="size-4" />
        </Button>
        <div className="ml-auto flex gap-2">
          {onDownloadFull && (
            <Button size="sm" variant="ghost" disabled={downloading} onClick={onDownloadFull}>
              <Download className="mr-1 size-4" />
              {t("Download full logs")}
            </Button>
          )}
          <Button
            size="sm"
            variant="ghost"
            onClick={() => saveBlob(new Blob([output], { type: "text/plain;charset=utf-8" }), `job-${jobId}.log`)}
          >
            <Download className="mr-1 size-4" />
            {t("Download loaded segment")}
          </Button>
          <Button size="sm" variant="ghost" onClick={() => void copy()}>
            <Copy className="mr-1 size-4" />
            {t("Copy")}
          </Button>
          <Button size="sm" variant={following ? "secondary" : "outline"} onClick={() => setFollowing(!following)}>
            <RefreshCw className="mr-1 size-4" />
            {t(following ? "Following output" : "Resume following")}
          </Button>
        </div>
      </div>
      {copyMessage && (
        <p role="status" className="px-3 py-1 text-xs">
          {copyMessage}
        </p>
      )}
      <div
        ref={viewport}
        className="relative h-[480px] overflow-auto ci-log font-mono text-xs"
        onScroll={(event) => {
          const view = event.currentTarget;
          if (view.scrollHeight - view.scrollTop - view.clientHeight > 80) setFollowing(false);
        }}
        tabIndex={0}
        aria-label={t("Job logs")}
      >
        {!output ? (
          <p className="p-5 text-muted-foreground">{t("No stored output yet.")}</p>
        ) : (
          <div style={{ height: virtualizer.getTotalSize(), width: "100%", position: "relative" }}>
            {virtualizer.getVirtualItems().map((row) => (
              <div
                key={row.key}
                className={`absolute left-0 top-0 flex min-w-full whitespace-pre leading-[22px] ${search && lines[row.index].toLowerCase().includes(search.toLowerCase()) ? "ci-log-match" : ""}`}
                style={{ height: row.size, transform: `translateY(${row.start}px)` }}
              >
                <span className="sticky left-0 mr-4 inline-block w-12 shrink-0 select-none ci-log-line-number px-2 text-right text-muted-foreground">
                  {row.index + 1}
                </span>
                <span className="pr-5">{lines[row.index]}</span>
              </div>
            ))}
          </div>
        )}
      </div>
      {!following && (
        <div className="border-t border-border p-2 text-center">
          <Button size="sm" variant="outline" onClick={() => setFollowing(true)}>
            <ArrowDown className="mr-1 size-4" />
            {t("Resume following")}
          </Button>
        </div>
      )}
    </div>
  );
};
