import { useEffect, useState } from "react";
import { useOne, useUpdate } from "@refinedev/core";
import { AtSign, KeyRound, ShieldCheck, UserRound } from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { useI18n } from "@/lib/i18n";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";
import type { UserView } from "@/pages/types";

export function AppProfilePage(): JSX.Element {
  const { t } = useI18n();
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const profileQuery = useOne<UserView>({ resource: "profile", id: "me" });
  const user = profileQuery.result ?? null;
  const { mutate: updateProfile, mutation: { isPending: isUpdating } } = useUpdate<UserView>();

  useEffect(() => {
    if (user) {
      setUsername(user.username);
      setEmail(user.email);
    }
  }, [user]);

  const loadError = profileQuery.query.error instanceof Error ? profileQuery.query.error.message : null;

  const submitProfile = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setError(null);
    setMessage(null);
    updateProfile(
      { resource: "profile", id: "me", values: { username, email, password: password || undefined } },
      {
        onSuccess: () => {
          setPassword("");
          setMessage(t("Profile updated."));
          void profileQuery.query.refetch();
        },
        onError: (updateError) => {
          setError(updateError instanceof Error ? updateError.message : t("Update failed"));
        },
      },
    );
  };

  return (
    <div className="flex flex-col gap-4 page-enter">
      <header className="flex flex-col gap-3 border-b border-border/70 pb-4 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="mb-1 text-xs font-medium text-muted-foreground">{t("My Account")}</p>
          <h1 className="text-xl font-semibold tracking-tight">{t("Profile")}</h1>
          <p className="mt-1 text-sm text-muted-foreground">{t("Current account information and access status.")}</p>
        </div>
        <Badge variant={user?.is_super_admin ? "default" : "secondary"} className="w-fit">
          {user?.is_super_admin ? t("Super Admin") : t("User")}
        </Badge>
      </header>

      {loadError ? <Alert variant="destructive"><AlertDescription>{loadError}</AlertDescription></Alert> : null}

      <div className="grid gap-4 lg:grid-cols-[minmax(240px,0.65fr)_minmax(0,1.35fr)]">
        <Card className="overflow-hidden shadow-sm">
          <CardHeader className="border-b border-border/70 p-4 sm:px-5">
            <CardTitle className="text-base">{t("My Account")}</CardTitle>
            <CardDescription>{t("Identity and quick navigation.")}</CardDescription>
          </CardHeader>
          <CardContent className="p-0">
            {profileQuery.query.isLoading ? <ProfileSkeleton /> : (
              <dl className="divide-y divide-border/70">
                <ProfileDetail icon={UserRound} label={t("Username")} value={user?.username ?? "--"} />
                <ProfileDetail icon={AtSign} label={t("Email")} value={user?.email ?? "--"} />
                <ProfileDetail icon={ShieldCheck} label={t("Account Status")} value={user?.status ?? "--"} />
                <ProfileDetail icon={KeyRound} label={t("Access role")} value={user?.is_super_admin ? t("Super Admin") : t("User")} />
              </dl>
            )}
          </CardContent>
        </Card>

        <Card className="overflow-hidden shadow-sm">
          <CardHeader className="border-b border-border/70 p-4 sm:px-5">
            <CardTitle className="text-base">{t("Edit Profile")}</CardTitle>
            <CardDescription>{t("Update username, email, and optionally change your password.")}</CardDescription>
          </CardHeader>
          <CardContent className="p-4 sm:p-5">
            <form className="grid gap-4 md:grid-cols-2" onSubmit={submitProfile}>
              <div className="space-y-2">
                <Label htmlFor="profile-username">{t("Username")}</Label>
                <Input id="profile-username" value={username} onChange={(event) => setUsername(event.target.value)} />
              </div>
              <div className="space-y-2">
                <Label htmlFor="profile-email">{t("Email")}</Label>
                <Input id="profile-email" value={email} onChange={(event) => setEmail(event.target.value)} />
              </div>
              <div className="space-y-2 md:col-span-2">
                <Label htmlFor="profile-password">{t("New Password (Optional)")}</Label>
                <Input id="profile-password" type="password" value={password} onChange={(event) => setPassword(event.target.value)} />
              </div>
              <div className="flex items-center justify-between gap-3 border-t border-border/70 pt-4 md:col-span-2">
                <p className="text-xs text-muted-foreground">{t("Current account information and access status.")}</p>
                <Button type="submit" disabled={isUpdating || profileQuery.query.isLoading}>
                  {isUpdating ? `${t("Save")}...` : t("Save Changes")}
                </Button>
              </div>
            </form>

            <div className="mt-4 space-y-2" aria-live="polite">
              {message ? <Alert className="border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300"><AlertDescription>{message}</AlertDescription></Alert> : null}
              {error ? <Alert variant="destructive"><AlertDescription>{error}</AlertDescription></Alert> : null}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

function ProfileDetail({ icon: Icon, label, value }: { icon: LucideIcon; label: string; value: string }): JSX.Element {
  return (
    <div className="flex items-center gap-3 px-4 py-3 sm:px-5">
      <Icon className="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
      <dt className="min-w-24 text-xs text-muted-foreground">{label}</dt>
      <dd className="min-w-0 truncate text-sm font-medium">{value}</dd>
    </div>
  );
}

function ProfileSkeleton(): JSX.Element {
  return <div className="space-y-px p-3">{[0, 1, 2, 3].map((item) => <Skeleton key={item} className="h-11 w-full" />)}</div>;
}
