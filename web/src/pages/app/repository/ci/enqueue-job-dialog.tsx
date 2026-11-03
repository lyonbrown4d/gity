import { useState } from "react";
import { useCustomMutation } from "@refinedev/core";
import { FormDialog } from "@/components/common/form-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { extractErrorMessage } from "../issues-utils";
interface Props {
  open: boolean;
  onClose: () => void;
  repoId: string;
  onCreated: () => void;
  t: (key: string) => string;
}
export const EnqueueJobDialog = ({ open, onClose, repoId, onCreated, t }: Props) => {
  const [kind, setKind] = useState("script");
  const [payload, setPayload] = useState('{"script":["echo hello"]}');
  const [attempts, setAttempts] = useState("3");
  const [error, setError] = useState<string | null>(null);
  const { mutateAsync, mutation } = useCustomMutation();
  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    if (mutation.isPending) return;
    try {
      if (kind === "script") JSON.parse(payload);
      await mutateAsync({
        url: `/projects/${repoId}/jobs`,
        method: "post",
        values: { kind, payload: kind === "script" ? payload : "", max_attempts: Number(attempts) },
      });
      setError(null);
      onCreated();
      onClose();
    } catch (failure) {
      setError(extractErrorMessage(failure));
    }
  };
  return (
    <FormDialog
      open={open}
      title={t("New job")}
      onClose={() => {
        if (!mutation.isPending) onClose();
      }}
    >
      <form className="space-y-4" onSubmit={(event) => void submit(event)}>
        <label className="block space-y-2 text-sm">
          {t("Kind")}
          <select
            className="h-9 w-full rounded-md border border-input bg-background px-2"
            value={kind}
            onChange={(event) => setKind(event.target.value)}
          >
            <option value="script">script</option>
            <option value="noop">noop</option>
          </select>
        </label>
        {kind === "script" && (
          <label className="block space-y-2 text-sm">
            {t("Job payload (JSON)")}
            <Textarea
              className="min-h-32 font-mono text-xs"
              value={payload}
              onChange={(event) => setPayload(event.target.value)}
              required
            />
          </label>
        )}
        <label className="block space-y-2 text-sm">
          {t("Max attempts")}
          <Input
            type="number"
            min={1}
            max={10}
            value={attempts}
            onChange={(event) => setAttempts(event.target.value)}
            required
          />
        </label>
        {error && (
          <p role="alert" className="text-sm text-destructive">
            {error}
          </p>
        )}
        <Button type="submit" disabled={mutation.isPending}>
          {t(mutation.isPending ? "Creating..." : "Enqueue job")}
        </Button>
      </form>
    </FormDialog>
  );
};
