import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

/** Horizontal action bar used above lists and inside section headers. */
export const Toolbar = ({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}): JSX.Element => <div className={cn("gity-toolbar", className)}>{children}</div>;
