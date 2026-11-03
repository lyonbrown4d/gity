import { useId, type ReactNode } from "react";
import {
  Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger,
} from "@/components/ui/dialog";

interface FormDialogProps {
  open: boolean;
  title: ReactNode;
  description?: ReactNode;
  trigger?: ReactNode;
  busy?: boolean;
  onOpen?: () => void;
  onClose: () => void;
  children: ReactNode;
}

export const FormDialog = ({ open, title, description, trigger, busy = false, onOpen, onClose, children }: FormDialogProps): JSX.Element => {
  const descriptionId = useId();
  return (
  <Dialog open={open} onOpenChange={(nextOpen) => {
    if (nextOpen) onOpen?.();
    else if (!busy) onClose();
  }}>
    {trigger ? <DialogTrigger asChild>{trigger}</DialogTrigger> : null}
    <DialogContent
      className={`max-h-[calc(100dvh-2rem)] w-[calc(100%-2rem)] max-w-xl overflow-y-auto motion-reduce:animate-none motion-reduce:transition-none ${busy ? "[&>button:last-child]:hidden" : ""}`}
      aria-describedby={description ? descriptionId : undefined}
      aria-busy={busy || undefined}
      onInteractOutside={(event) => { if (busy) event.preventDefault(); }}
      onEscapeKeyDown={(event) => { if (busy) event.preventDefault(); }}
    >
      <DialogHeader>
        <DialogTitle>{title}</DialogTitle>
        {description ? <DialogDescription id={descriptionId}>{description}</DialogDescription> : null}
      </DialogHeader>
      {children}
    </DialogContent>
  </Dialog>
  );
};
