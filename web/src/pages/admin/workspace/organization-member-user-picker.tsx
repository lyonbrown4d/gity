import { useState } from "react";
import { Check, ChevronsUpDown } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList } from "@/components/ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import type { UserView } from "@/pages/types";

interface Props {
  users: UserView[];
  value: string;
  loading: boolean;
  disabled?: boolean;
  t: (key: string) => string;
  onChange: (userId: string) => void;
}

export const OrganizationMemberUserPicker = ({ users, value, loading, disabled, t, onChange }: Props): JSX.Element => {
  const [open, setOpen] = useState(false);
  const selected = users.find((user) => user.id === value);
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          type="button"
          variant="outline"
          role="combobox"
          aria-label={t("User")}
          aria-required="true"
          aria-expanded={open}
          disabled={disabled || loading}
          className="w-full justify-between font-normal"
        >
          <span className="truncate">{selected ? selected.display_name || selected.username : t("Select user")}</span>
          <ChevronsUpDown className="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
        </Button>
      </PopoverTrigger>
      <PopoverContent align="start" className="w-[var(--radix-popover-trigger-width)] min-w-64 p-0">
        <Command
          loop
          filter={(id, search) => {
            const user = users.find((item) => item.id === id);
            if (!user) return 0;
            const searchable = `${user.display_name ?? ""} ${user.username} ${user.email}`.toLocaleLowerCase();
            return searchable.includes(search.toLocaleLowerCase()) ? 1 : 0;
          }}
        >
          <CommandInput placeholder={t("Search members")} aria-label={t("Search members")} />
          <CommandList>
            <CommandEmpty>{loading ? t("Loading...") : t("No matches found.")}</CommandEmpty>
            <CommandGroup>
              {users.map((user) => (
                <CommandItem
                  key={user.id}
                  value={user.id}
                  onSelect={() => {
                    onChange(user.id);
                    setOpen(false);
                  }}
                >
                  <Check className={`size-4 ${value === user.id ? "opacity-100" : "opacity-0"}`} aria-hidden="true" />
                  <span className="min-w-0 flex-1">
                    <span className="block truncate">{user.display_name || user.username}</span>
                    <span className="block truncate text-xs text-muted-foreground">{user.email}</span>
                  </span>
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
};
