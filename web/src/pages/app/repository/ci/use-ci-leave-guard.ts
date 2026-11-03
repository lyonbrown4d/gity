import { useState } from "react";
import { useBlocker } from "react-router";
export const useCILeaveGuard = (dirty: boolean, onDiscard: () => void) => {
  const blocker = useBlocker(dirty);
  const [localAction, setLocalAction] = useState<(() => void) | null>(null);
  return {
    open: Boolean(localAction) || blocker.state === "blocked",
    leave: (action: () => void) => {
      if (dirty) setLocalAction(() => action);
      else action();
    },
    keep: () => {
      setLocalAction(null);
      if (blocker.state === "blocked") blocker.reset();
    },
    discard: () => {
      onDiscard();
      if (blocker.state === "blocked") blocker.proceed();
      else localAction?.();
      setLocalAction(null);
    },
  };
};
