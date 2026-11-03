import type { ReactNode } from "react";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";

interface Props {
  label: string;
  value: string;
  onChange: (value: string) => void;
  items: { value: string; label: ReactNode }[];
  children: ReactNode;
}

/** Underlined page sections with the same keyboard and focus behavior everywhere. */
export const SectionTabs = ({ label, value, onChange, items, children }: Props) => (
  <Tabs value={value} onValueChange={onChange} activationMode="manual">
    <TabsList
      aria-label={label}
      className="h-auto w-full justify-start gap-2 overflow-x-auto rounded-none border-b border-border bg-transparent p-0"
    >
      {items.map((item) => (
        <TabsTrigger
          key={item.value}
          value={item.value}
          className="min-h-11 shrink-0 rounded-none border-b-2 border-transparent px-4 shadow-none data-[state=active]:border-primary data-[state=active]:bg-transparent data-[state=active]:text-foreground data-[state=active]:shadow-none"
        >
          {item.label}
        </TabsTrigger>
      ))}
    </TabsList>
    {children}
  </Tabs>
);

export const SectionTabPanel = ({ value, children }: { value: string; children: ReactNode }) => (
  <TabsContent value={value} forceMount className="m-0 min-w-0 data-[state=inactive]:hidden">
    {children}
  </TabsContent>
);
