import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

/** Horizontal action bar used above lists and inside section headers. */
export function Toolbar({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}): JSX.Element {
  return <div className={cn("gity-toolbar", className)}>{children}</div>;
}
