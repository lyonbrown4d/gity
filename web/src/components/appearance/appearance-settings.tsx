import { RotateCcw, SwatchBook } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle, SheetTrigger } from "@/components/ui/sheet";
import { useI18n } from "@/lib/i18n";
import { palettes, type Palette, type ThemeMode } from "@/lib/appearance/palettes";
import { useTheme } from "@/lib/theme";

const modes: ThemeMode[] = ["system", "light", "dark"];
const radiusOptions = [4, 8, 12] as const;

export const AppearanceSettings = ({ compact = false }: { compact?: boolean }): JSX.Element => {
  const { appearance, setTheme, updateAppearance, resetAppearance } = useTheme();
  const { t } = useI18n();
  const selectedPalette = palettes[appearance.palette];
  const colorInputs = [
    { key: "accent", label: t("Accent color"), fallback: selectedPalette.accent },
    { key: "background", label: t("Page background"), fallback: selectedPalette.canvas },
    { key: "sidebar", label: t("Sidebar background"), fallback: selectedPalette.sidebar },
  ] as const;

  return <Sheet>
    <SheetTrigger asChild>
      <Button type="button" variant="outline" size={compact ? "sm" : "default"} aria-label={t("Palette settings")} title={t("Palette settings")}>
        <SwatchBook aria-hidden="true" />{compact ? null : t("Palette")}
      </Button>
    </SheetTrigger>
    <SheetContent className="flex w-full flex-col gap-0 overflow-y-auto p-0 sm:max-w-md">
      <SheetHeader className="border-b border-border px-5 py-5 text-left">
        <SheetTitle>{t("Appearance settings")}</SheetTitle>
        <SheetDescription>{t("Choose how Gity looks. Changes apply and save in this browser immediately.")}</SheetDescription>
      </SheetHeader>
      <div className="flex flex-col gap-6 px-5 py-5">
        <fieldset className="space-y-3">
          <legend className="text-sm font-semibold">{t("Color mode")}</legend>
          <div className="grid grid-cols-3 gap-2">
            {modes.map((mode) => <Button key={mode} type="button" size="sm" variant={appearance.mode === mode ? "default" : "outline"} aria-pressed={appearance.mode === mode} onClick={() => setTheme(mode)}>{t(mode === "system" ? "System" : mode === "light" ? "Light" : "Dark")}</Button>)}
          </div>
        </fieldset>

        <fieldset className="space-y-3">
          <legend className="text-sm font-semibold">{t("Color palette")}</legend>
          <div className="grid grid-cols-2 gap-2">
            {(Object.entries(palettes) as Array<[Palette, typeof palettes[Palette]]>).map(([palette, colors]) => <button key={palette} type="button" aria-pressed={appearance.palette === palette} onClick={() => updateAppearance({ palette })} className={`flex items-center gap-3 rounded-md border p-3 text-left text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring ${appearance.palette === palette ? "border-primary bg-accent" : "border-border hover:bg-muted"}`}>
              <span className="flex size-8 shrink-0 overflow-hidden rounded-full border border-border"><span className="flex-1" style={{ backgroundColor: colors.accent }} /><span className="flex-1" style={{ backgroundColor: colors.canvas }} /></span>
              <span className="font-medium">{t(colors.label)}</span>
            </button>)}
          </div>
        </fieldset>

        <fieldset className="space-y-3">
          <legend className="text-sm font-semibold">{t("Custom colors")}</legend>
          <div className="space-y-3">
            {colorInputs.map(({ key, label, fallback }) => <div key={key} className="flex items-center justify-between gap-4">
              <Label htmlFor={`appearance-${key}`}>{label}</Label>
              <Input id={`appearance-${key}`} aria-label={label} type="color" value={appearance[key] || fallback} onChange={(event) => updateAppearance({ [key]: event.currentTarget.value })} className="h-9 w-16 cursor-pointer p-1" />
            </div>)}
          </div>
        </fieldset>

        <fieldset className="space-y-3">
          <legend className="text-sm font-semibold">{t("Corner radius")}</legend>
          <div className="grid grid-cols-3 gap-2">
            {radiusOptions.map((radius) => <Button key={radius} type="button" size="sm" variant={appearance.radius === radius ? "default" : "outline"} aria-pressed={appearance.radius === radius} onClick={() => updateAppearance({ radius })}>{radius}px</Button>)}
          </div>
        </fieldset>

        <div className="flex items-start justify-between gap-4 border-t border-border pt-4">
          <p className="text-xs leading-relaxed text-muted-foreground">{t("Preferences are saved automatically in this browser.")}</p>
          <Button type="button" variant="outline" size="sm" onClick={resetAppearance} className="shrink-0">
            <RotateCcw aria-hidden="true" />{t("Reset appearance")}
          </Button>
        </div>
      </div>
    </SheetContent>
  </Sheet>;
};
