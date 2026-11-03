import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import type { RepositoryBranchView } from "@/pages/types";

export const BranchSelect = ({
  id,
  label,
  value,
  branches,
  placeholder,
  onChange,
}: {
  id: string;
  label: string;
  value: string;
  branches: RepositoryBranchView[];
  placeholder: string;
  onChange: (value: string) => void;
}) => (
  <div className="flex flex-col gap-1">
    <Label className="text-xs font-medium text-muted-foreground" htmlFor={id}>
      {label}
    </Label>
    <Select value={value} onValueChange={onChange}>
      <SelectTrigger id={id}>
        <SelectValue placeholder={placeholder} />
      </SelectTrigger>
      <SelectContent>
        <SelectGroup>
          {branches.map((branch) => (
            <SelectItem key={branch.name} value={branch.name}>
              {branch.name}
            </SelectItem>
          ))}
        </SelectGroup>
      </SelectContent>
    </Select>
  </div>
);
