import { useEffect, useState } from "react";
import { useOne, useUpdate } from "@refinedev/core";
import { AtSign, KeyRound, ShieldCheck, UserRound } from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { useI18n } from "@/lib/i18n";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";
import { Section } from "@/components/surface/section";
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
          <p className="mt-1 max-w-2xl text-sm text-muted-foreground">{t("Current account information and access status.")}</p>
        </div>
        <Badge variant={user?.is_super_admin ? "default" : "secondary"} className="w-fit">
          {user?.is_super_admin ? t("Super Admin") : t("User")}
        </Badge>
      </header>

      {loadError ? <Alert variant="destructive"><AlertDescription>{loadError}</AlertDescription></Alert> : null}

      <div className="grid items-start gap-4 lg:grid-cols-[minmax(240px,0.6fr)_minmax(0,1.4fr)]">
        <Section title={t("My Account")} description={t("Identity and quick navigation.")} flush>
          {profileQuery.query.isLoading ? <ProfileSkeleton /> : (
            <dl className="gity-list">
              <ProfileDetail icon={UserRound} label={t("Username")} value={user?.username ?? "--"} />
              <ProfileDetail icon={AtSign} label={t("Email")} value={user?.email ?? "--"} />
              <ProfileDetail icon={ShieldCheck} label={t("Account Status")} value={user?.status ?? "--"} />
              <ProfileDetail icon={KeyRound} label={t("Access role")} value={user?.is_super_admin ? t("Super Admin") : t("User")} />
            </dl>
          )}
        </Section>

        <Section
          title={t("Edit Profile")}
          description={t("Update username, email, and optionally change your password.")}
          actions={(
            <Button type="submit" form="profile-form" size="sm" disabled={isUpdating || profileQuery.query.isLoading}>
              {isUpdating ? `${t("Save")}...` : t("Save Changes")}
            </Button>
          )}
        >
          <form id="profile-form" className="grid gap-3 md:grid-cols-2" onSubmit={submitProfile}>
            <div className="space-y-1.5">
              <Label htmlFor="profile-username" className="text-xs">{t("Username")}</Label>
              <Input id="profile-username" value={username} onChange={(event) => setUsername(event.target.value)} />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="profile-email" className="text-xs">{t("Email")}</Label>
              <Input id="profile-email" value={email} onChange={(event) => setEmail(event.target.value)} />
            </div>
            <div className="space-y-1.5 md:col-span-2">
              <Label htmlFor="profile-password" className="text-xs">{t("New Password (Optional)")}</Label>
              <Input id="profile-password" type="password" value={password} onChange={(event) => setPassword(event.target.value)} />
            </div>
          </form>

          <div className="mt-3 space-y-2" aria-live="polite">
            {message ? <Alert className="border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300"><AlertDescription>{message}</AlertDescription></Alert> : null}
            {error ? <Alert variant="destructive"><AlertDescription>{error}</AlertDescription></Alert> : null}
          </div>
        </Section>
      </div>
    </div>
  );
}

function ProfileDetail({ icon: Icon, label, value }: { icon: LucideIcon; label: string; value: string }): JSX.Element {
  return (
    <div className="gity-list-row">
      <Icon className="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
      <dt className="w-24 shrink-0 text-xs text-muted-foreground">{label}</dt>
      <dd className="min-w-0 truncate text-[13px] font-medium">{value}</dd>
    </div>
  );
}

function ProfileSkeleton(): JSX.Element {
  return <div className="space-y-px p-3">{[0, 1, 2, 3].map((item) => <Skeleton key={item} className="h-10 w-full" />)}</div>;
}
