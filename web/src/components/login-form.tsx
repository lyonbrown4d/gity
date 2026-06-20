import { useState } from "react";
import { ArrowRightLeft, CheckCircle2, GitBranch, LockKeyhole, Workflow } from "lucide-react";
import { cn } from "@/lib/utils";
import { useI18n } from "@/lib/i18n";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ProductCodePanel, ProductFeatureList } from "@/components/ui/product";

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
      <Card className="gity-hero overflow-hidden card-enter">
        <CardContent className="relative z-10 grid p-0 lg:grid-cols-[0.9fr_1.1fr]">
          <form
            className="bg-card/80 p-6 backdrop-blur-xl md:p-8 lg:p-10"
            onSubmit={(event) => {
              event.preventDefault();
              onLoginSubmit({ username, password });
            }}
          >
            <div className="flex flex-col gap-7">
              <div className="flex flex-col gap-3">
                <div className="flex flex-col gap-2">
                  <h1 className="text-3xl font-semibold md:text-4xl">{t("Welcome back")}</h1>
                  <p className="text-balance text-sm leading-6 text-muted-foreground">
                    {t("Sign in to manage projects and organizations.")}
                  </p>
                </div>
              </div>
              <div className="grid gap-3 rounded-lg border border-border/70 bg-background/70 p-3 text-xs text-muted-foreground">
                <div className="flex items-center gap-2">
                  <LockKeyhole className="size-4 text-primary" />
                  <span>{t("Refine-driven routing and auth flow")}</span>
                </div>
                <div className="flex items-center gap-2">
                  <GitBranch className="size-4 text-primary" />
                  <span>{t("Project source view and clone information.")}</span>
                </div>
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
                  className="bg-background/80"
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
                  className="bg-background/80"
                />
              </div>
              {error ? (
                <Alert variant="destructive">
                  <AlertDescription>{error}</AlertDescription>
                </Alert>
              ) : null}
              <Button type="submit" className="h-11 w-full action-pop" disabled={loading}>
                {loading ? t("Signing in...") : t("Sign in")}
              </Button>
            </div>
          </form>
          <div className="relative hidden min-h-[620px] overflow-hidden bg-foreground text-background lg:block">
            <div className="absolute inset-0 opacity-45 gity-soft-grid" />
            <div className="absolute inset-0 bg-[linear-gradient(135deg,hsl(var(--primary)/0.34),transparent_42%)]" />
            <div className="relative flex h-full flex-col justify-between p-10">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div className="flex size-11 items-center justify-center rounded-lg bg-background text-foreground">
                    <ArrowRightLeft className="size-5" />
                  </div>
                  <div>
                    <p className="text-sm font-semibold">Gity Standalone</p>
                    <p className="text-xs text-background/60">GitLab-like workspace</p>
                  </div>
                </div>
                <div className="rounded-md border border-background/20 px-3 py-1 text-xs text-background/70">
                  same-stack backend
                </div>
              </div>

              <div className="flex flex-col gap-6">
                <div className="flex max-w-md flex-col gap-3">
                  <p className="text-xs font-semibold uppercase text-background/55">Project OS</p>
                  <h2 className="text-4xl font-semibold leading-tight">
                    Ship code, review changes, run pipelines.
                  </h2>
                  <p className="text-sm leading-6 text-background/65">
                    Unified project, issue, wiki, runner, and package workflows on a compact Go service.
                  </p>
                </div>

                <ProductFeatureList
                  items={[
                    { icon: CheckCircle2, text: "Project, issue, MR, wiki" },
                    { icon: Workflow, text: "Runner jobs and CI pipeline control" },
                    { icon: GitBranch, text: "Code search and branch operations" },
                  ]}
                />
              </div>

              <ProductCodePanel title="pipeline.plano" className="border-background/10 bg-background/10 text-background">
                {`stage "test" {
  run "go test ./..."
  artifact "coverage.out"
}`}
              </ProductCodePanel>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
