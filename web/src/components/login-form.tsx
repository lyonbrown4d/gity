import { useState } from "react";
import { Braces, GitBranch, LockKeyhole, Workflow } from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { cn } from "@/lib/utils";
import { useI18n } from "@/lib/i18n";
import { Alert, AlertDescription } from "@/components/ui/alert";
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
}: LoginFormProps): JSX.Element {
  const { t } = useI18n();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");

  return (
    <div className={cn("page-enter", className)} {...props}>
      <Card className="overflow-hidden border-border/80 shadow-xl shadow-foreground/5">
        <CardContent className="grid p-0 lg:grid-cols-[minmax(260px,0.75fr)_minmax(0,1.25fr)]">
          <aside className="relative hidden overflow-hidden border-r border-border/70 bg-muted/45 p-6 lg:flex lg:flex-col lg:justify-between">
            <div className="absolute inset-0 gity-soft-grid opacity-35" aria-hidden="true" />
            <div className="relative">
              <div className="flex items-center gap-3">
                <span className="flex size-9 items-center justify-center rounded-md border bg-background shadow-sm">
                  <Braces className="size-4 text-primary" aria-hidden="true" />
                </span>
                <div>
                  <p className="text-sm font-semibold">Gity</p>
                  <p className="text-xs text-muted-foreground">Code Canvas</p>
                </div>
              </div>

              <div className="mt-10 border-y border-border/70 py-4">
                <p className="text-xs font-medium text-muted-foreground">{t("Workspace")}</p>
                <p className="mt-2 text-lg font-semibold leading-snug">
                  {t("Project activity, organization scope, and account access in one workspace.")}
                </p>
              </div>
            </div>

            <div className="relative divide-y divide-border/70 border-y border-border/70 text-sm">
              <CanvasLine icon={GitBranch} label={t("Projects")} value={t("Code")} />
              <CanvasLine icon={Workflow} label={t("CI/CD")} value={t("Operate")} />
              <CanvasLine icon={LockKeyhole} label={t("Access role")} value={t("Account Status")} />
            </div>
          </aside>

          <form
            className="bg-card p-6 sm:p-8 lg:p-10"
            onSubmit={(event) => {
              event.preventDefault();
              onLoginSubmit({ username, password });
            }}
          >
            <div className="mx-auto flex max-w-sm flex-col gap-6">
              <div>
                <div className="mb-5 flex items-center gap-2 lg:hidden">
                  <span className="flex size-8 items-center justify-center rounded-md border bg-muted">
                    <Braces className="size-4 text-primary" aria-hidden="true" />
                  </span>
                  <span className="text-sm font-semibold">Gity</span>
                </div>
                <p className="text-xs font-medium text-muted-foreground">{t("User Workspace")}</p>
                <h1 className="mt-1 text-2xl font-semibold tracking-tight">{t("Welcome back")}</h1>
                <p className="mt-2 text-sm leading-6 text-muted-foreground">
                  {t("Sign in to manage projects and organizations.")}
                </p>
              </div>

              <div className="grid gap-2">
                <Label htmlFor="username">{t("Username / Email")}</Label>
                <Input
                  id="username"
                  type="text"
                  autoComplete="username"
                  placeholder="admin@example.com"
                  value={username}
                  onChange={(event) => setUsername(event.target.value)}
                  required
                  autoFocus
                />
              </div>
              <div className="grid gap-2">
                <div className="flex items-center justify-between gap-3">
                  <Label htmlFor="password">{t("Password")}</Label>
                  <LockKeyhole className="size-3.5 text-muted-foreground" aria-hidden="true" />
                </div>
                <Input
                  id="password"
                  type="password"
                  autoComplete="current-password"
                  value={password}
                  onChange={(event) => setPassword(event.target.value)}
                  required
                />
              </div>
              {error ? (
                <Alert variant="destructive" aria-live="polite">
                  <AlertDescription>{error}</AlertDescription>
                </Alert>
              ) : null}
              <Button type="submit" className="h-10 w-full" disabled={loading}>
                {loading ? t("Signing in...") : t("Sign in")}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}

function CanvasLine({ icon: Icon, label, value }: { icon: LucideIcon; label: string; value: string }): JSX.Element {
  return (
    <div className="flex items-center gap-3 py-3">
      <Icon className="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
      <span className="font-medium">{label}</span>
      <span className="ml-auto text-xs text-muted-foreground">{value}</span>
    </div>
  );
}
