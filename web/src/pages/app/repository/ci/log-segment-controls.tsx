import { Button } from "@/components/ui/button";
interface Props {
  offset: number;
  nextOffset: number;
  total: number;
  hasMore: boolean;
  pending: boolean;
  onBeginning: () => void;
  onPrevious: () => void;
  onNext: () => void;
  onLatest: () => void;
  t: (key: string) => string;
}
export const LogSegmentControls = ({
  offset,
  nextOffset,
  total,
  hasMore,
  pending,
  onBeginning,
  onPrevious,
  onNext,
  onLatest,
  t,
}: Props) => <div className="flex flex-wrap items-center justify-between gap-3 rounded-md border border-border bg-card p-3">
      <p className="text-xs text-muted-foreground">
        {t("Loaded bytes")}: {offset.toLocaleString()}–{nextOffset.toLocaleString()} / {total.toLocaleString()}
      </p>
      <div className="flex flex-wrap gap-2">
        <Button size="sm" variant="outline" disabled={pending || offset === 0} onClick={onBeginning}>
          {t("Beginning")}
        </Button>
        <Button size="sm" variant="outline" disabled={pending || offset === 0} onClick={onPrevious}>
          {t("Previous segment")}
        </Button>
        <Button size="sm" variant="outline" disabled={pending || !hasMore} onClick={onNext}>
          {t("Next segment")}
        </Button>
        <Button size="sm" variant="outline" disabled={pending} onClick={onLatest}>
          {t("Latest output")}
        </Button>
      </div>
    </div>;
