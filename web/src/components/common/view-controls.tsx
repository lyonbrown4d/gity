import { Globe2, Moon, Sun } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useI18n } from "@/lib/i18n";
import { useTheme } from "@/lib/theme";

interface ViewControlsProps {
  compact?: boolean;
}

export function ViewControls({ compact = false }: ViewControlsProps): JSX.Element {
  const { theme, toggleTheme } = useTheme();
  const { locale, toggleLocale, t } = useI18n();

  return (
    <div className="flex items-center gap-2">
      <Button
        type="button"
        variant="outline"
        size={compact ? "sm" : "default"}
        onClick={toggleTheme}
        title={t("Switch theme")}
        aria-label={t("Switch theme")}
        className="action-pop"
      >
        {theme === "dark" ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
      </Button>
      <Button
        type="button"
        variant="outline"
        size={compact ? "sm" : "default"}
        onClick={toggleLocale}
        title={t("Switch language")}
        aria-label={t("Switch language")}
        className="gap-1 action-pop"
      >
        <Globe2 className="h-4 w-4" />
        <span className="text-xs font-semibold">{locale === "en" ? "EN" : "中"}</span>
      </Button>
    </div>
  );
}
