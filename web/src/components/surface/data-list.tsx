import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

/** Divided list used for dense rows instead of stacked cards. */
export function DataList({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}): JSX.Element {
  return <ul className={cn("gity-list", className)}>{children}</ul>;
}

export function DataRow({
  children,
  className,
  onClick,
}: {
  children: ReactNode;
  className?: string;
  onClick?: () => void;
}): JSX.Element {
  return (
    <li className={cn("gity-list-row", onClick && "cursor-pointer", className)} onClick={onClick}>
      {children}
    </li>
  );
}
