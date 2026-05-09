import type { ReactNode } from "react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";

interface ConfirmActionProps {
  children: ReactNode;
  title: ReactNode;
  description: ReactNode;
  confirmLabel: ReactNode;
  cancelLabel: ReactNode;
  verificationLabel?: ReactNode;
  verificationValue?: string;
  verificationPlaceholder?: string;
  onConfirm: (verification?: string) => void;
}

export function ConfirmAction({
  children,
  title,
  description,
  confirmLabel,
  cancelLabel,
  verificationLabel,
  verificationValue,
  verificationPlaceholder,
  onConfirm,
}: ConfirmActionProps): JSX.Element {
  const [open, setOpen] = useState(false);
  const [verification, setVerification] = useState("");
  const requiresVerification = typeof verificationValue === "string" && verificationValue.length > 0;
  const canConfirm = !requiresVerification || verification === verificationValue;

  const updateOpen = (nextOpen: boolean) => {
    setOpen(nextOpen);
    if (!nextOpen) {
      setVerification("");
    }
  };

  return (
    <Dialog open={open} onOpenChange={updateOpen}>
      <DialogTrigger asChild>{children}</DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
        {requiresVerification ? (
          <div className="space-y-2">
            <p className="text-sm text-muted-foreground">
              {verificationLabel}
            </p>
            <Input
              value={verification}
              placeholder={verificationPlaceholder ?? verificationValue}
              autoComplete="off"
              onChange={(event) => setVerification(event.target.value)}
            />
          </div>
        ) : null}
        <DialogFooter>
          <Button type="button" variant="outline" onClick={() => updateOpen(false)}>
            {cancelLabel}
          </Button>
          <Button
            type="button"
            variant="destructive"
            disabled={!canConfirm}
            onClick={() => {
              onConfirm(verification);
              updateOpen(false);
            }}
          >
            {confirmLabel}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
