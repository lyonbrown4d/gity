import { Package } from "lucide-react";

export const PackageStat = ({ label, value }: { label: string; value: number }) => (
  <div className="px-4 py-2.5">
    <p className="text-xs text-muted-foreground">{label}</p>
    <p className="mt-0.5 text-sm font-semibold">{value}</p>
  </div>
);

export const PackageEmptyState = ({ title, description }: { title: string; description: string }) => (
  <div className="flex flex-col items-center justify-center gap-2 text-center">
    <Package className="size-4 text-muted-foreground" />
    <div className="flex flex-col gap-1">
      <p className="text-sm font-medium">{title}</p>
      <p className="text-xs text-muted-foreground">{description}</p>
    </div>
  </div>
);
