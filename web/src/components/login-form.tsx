import { useState } from "react";
import { cn } from "@/lib/utils";
import { useI18n } from "@/lib/i18n";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface LoginFormProps extends React.ComponentProps<"div"> {
  onLoginSubmit: (credentials: { username: string; password: string }) => void;
  loading?: boolean;
  error?: string | null;
}

export function LoginForm({
  className,
  onLoginSubmit,
  loading = false,
  error = null,
  ...props
}: LoginFormProps) {
  const { t } = useI18n();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");

  return (
    <div className={cn("flex flex-col gap-6 page-enter", className)} {...props}>
      <Card className="overflow-hidden card-enter">
        <CardContent className="grid p-0 md:grid-cols-2">
          <form
            className="p-6 md:p-8"
            onSubmit={(event) => {
              event.preventDefault();
              onLoginSubmit({ username, password });
            }}
          >
            <div className="flex flex-col gap-6">
              <div className="flex flex-col items-center text-center">
                <h1 className="text-2xl font-bold">{t("Welcome back")}</h1>
                <p className="text-balance text-muted-foreground">
                  {t("Sign in to manage repositories and organizations.")}
                </p>
              </div>
              <div className="grid gap-2">
                <Label htmlFor="username">{t("Username / Email")}</Label>
                <Input
                  id="username"
                  type="text"
                  placeholder="admin@example.com"
                  value={username}
                  onChange={(event) => setUsername(event.target.value)}
                  required
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="password">{t("Password")}</Label>
                <Input
                  id="password"
                  type="password"
                  value={password}
                  onChange={(event) => setPassword(event.target.value)}
                  required
                />
              </div>
              {error ? <p className="text-sm text-red-600">{error}</p> : null}
              <Button type="submit" className="w-full action-pop" disabled={loading}>
                {loading ? t("Signing in...") : t("Sign in")}
              </Button>
            </div>
          </form>
          <div className="relative hidden bg-muted md:block">
            <div className="absolute inset-0 bg-[radial-gradient(circle_at_20%_20%,hsl(217_91%_60%/.25),transparent_45%),radial-gradient(circle_at_80%_80%,hsl(173_58%_40%/.2),transparent_40%)]" />
            <div className="absolute inset-x-6 bottom-6 rounded-lg border bg-background/80 p-4 backdrop-blur">
              <p className="text-sm font-medium">{t("Gity Standalone")}</p>
              <p className="mt-1 text-xs text-muted-foreground">
                {t("Unified workspace for auth, organization management, and repository operations.")}
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
