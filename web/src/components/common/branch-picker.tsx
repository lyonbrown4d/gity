import { useState } from "react";
import { Check, ChevronDown, GitBranch, ShieldCheck } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList } from "@/components/ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";

interface BranchOption {
  name: string;
  isProtected?: boolean;
}
interface Props {
  value: string;
  branches: BranchOption[];
  onChange: (branch: string) => void;
  t: (key: string) => string;
  disabled?: boolean;
}

export const BranchPicker = ({ value, branches, onChange, t, disabled }: Props) => {
  const [open, setOpen] = useState(false);
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          type="button"
          variant="outline"
          role="combobox"
          aria-expanded={open}
          aria-label={t("Branch")}
          disabled={disabled}
          className="min-h-11 max-w-64 sm:min-h-9 justify-between gap-2 font-mono text-xs"
        >
          <GitBranch className="size-4 shrink-0" />
          <span className="truncate">{value || t("Branch")}</span>
          <ChevronDown className="size-4 shrink-0 text-muted-foreground" />
        </Button>
      </PopoverTrigger>
      <PopoverContent align="start" className="w-72 max-w-[calc(100vw-2rem)] p-0">
        <Command loop filter={(name, search) => (name.toLowerCase().includes(search.toLowerCase()) ? 1 : 0)}>
          <CommandInput placeholder={t("Search branches")} aria-label={t("Search branches")} />
          <CommandList aria-label={t("Branches")}>
            <CommandEmpty>{t("No branches match.")}</CommandEmpty>
            <CommandGroup heading={t("Switch branch")}>
              {branches.map((branch) => (
                <CommandItem
                  key={branch.name}
                  value={branch.name}
                  onSelect={() => {
                    setOpen(false);
                    if (branch.name !== value) onChange(branch.name);
                  }}
                >
                  {branch.name === value ? (
                    <Check className="size-4 text-primary" />
                  ) : (
                    <GitBranch className="size-4 text-muted-foreground" />
                  )}
                  <span className="min-w-0 flex-1 truncate font-mono text-xs">{branch.name}</span>
                  {branch.isProtected && (
                    <span className="flex shrink-0 items-center gap-1 text-xs text-success">
                      <ShieldCheck className="size-3.5" />
                      {t("Protected branch")}
                    </span>
                  )}
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
};
