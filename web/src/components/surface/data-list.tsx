import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

/** Divided list used for dense rows instead of stacked cards. */
export const DataList = ({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}): JSX.Element => <ul className={cn("gity-list", className)}>{children}</ul>;

export const DataRow = ({
  children,
  className,
  onClick,
}: {
  children: ReactNode;
  className?: string;
  onClick?: () => void;
}): JSX.Element => <li className={cn("gity-list-row", onClick && "cursor-pointer", className)} onClick={onClick}>
      {children}
    </li>;
