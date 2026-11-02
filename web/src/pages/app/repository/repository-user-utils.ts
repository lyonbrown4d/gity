import type { UserView } from "@/pages/types";

export const formatUserLabel = (user: UserView | undefined, fallbackID = ""): string => {
  if (!user) {
    return fallbackID.trim() ? `#${fallbackID}` : "Unknown user";
  }
  const displayName = user.display_name?.trim();
  return displayName ? `${displayName} (@${user.username})` : `@${user.username}`;
};

export const uniqueStrings = (values: string[]): string[] => {
  return Array.from(new Set(values.filter((value) => value.trim().length > 0)));
};
