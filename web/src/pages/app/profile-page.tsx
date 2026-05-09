import { useEffect, useState } from "react";
import { useOne, useUpdate } from "@refinedev/core";
import { useI18n } from "@/lib/i18n";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { UserView } from "@/pages/types";

export function AppProfilePage(): JSX.Element {
  const { t } = useI18n();
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const profileQuery = useOne<UserView>({
    resource: "profile",
    id: "me",
  });
  const user = profileQuery.data?.data ?? null;
  const { mutate: updateProfile, isLoading: isUpdating } = useUpdate<UserView>();

  useEffect(() => {
    if (!user) {
      return;
    }
    setUsername(user.username);
    setEmail(user.email);
  }, [user]);

  const loadError = profileQuery.error instanceof Error ? profileQuery.error.message : null;

  return (
    <div className="space-y-4 page-enter">
      <Card className="card-enter">
        <CardHeader>
          <CardTitle>{t("Profile")}</CardTitle>
          <CardDescription>{t("Current account information and access status.")}</CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <div className="space-y-1">
            <p className="font-medium">{user?.username ?? "--"}</p>
            <p className="text-sm text-muted-foreground">{user?.email ?? "--"}</p>
          </div>
          <div className="flex items-center gap-2">
            <Badge variant="secondary">{user?.status ?? "--"}</Badge>
            {user?.is_super_admin ? <Badge>{t("Super Admin")}</Badge> : null}
          </div>
        </CardContent>
      </Card>

      <Card className="card-enter">
        <CardHeader>
          <CardTitle>{t("Edit Profile")}</CardTitle>
          <CardDescription>{t("Update username, email, and optionally change your password.")}</CardDescription>
        </CardHeader>
        <CardContent>
          <form
            className="grid gap-4 md:grid-cols-2"
            onSubmit={async (event) => {
              event.preventDefault();
              setError(null);
              setMessage(null);
              updateProfile(
                {
                  resource: "profile",
                  id: "me",
                  values: {
                    username,
                    email,
                    password: password || undefined,
                  },
                },
                {
                  onSuccess: () => {
                    setPassword("");
                    setMessage(t("Profile updated."));
                    void profileQuery.refetch();
                  },
                  onError: (e) => {
                    setError(e instanceof Error ? e.message : t("Update failed"));
                  },
                },
              );
            }}
          >
            <div className="space-y-2">
              <Label htmlFor="profile-username">{t("Username")}</Label>
              <Input
                id="profile-username"
                value={username}
                onChange={(event) => setUsername(event.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="profile-email">{t("Email")}</Label>
              <Input
                id="profile-email"
                value={email}
                onChange={(event) => setEmail(event.target.value)}
              />
            </div>
            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="profile-password">{t("New Password (Optional)")}</Label>
              <Input
                id="profile-password"
                type="password"
                value={password}
                onChange={(event) => setPassword(event.target.value)}
              />
            </div>
            <div className="md:col-span-2 flex items-center gap-2">
              <Button type="submit" disabled={isUpdating || profileQuery.isLoading}>
                {isUpdating ? `${t("Save")}...` : t("Save Changes")}
              </Button>
            </div>
          </form>

          <div className="mt-4 space-y-2">
            {message ? (
              <Alert className="border-emerald-500/30 bg-emerald-500/10 text-emerald-700">
                <AlertDescription>{message}</AlertDescription>
              </Alert>
            ) : null}
            {error ? (
              <Alert variant="destructive">
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            ) : null}
            {loadError ? (
              <Alert variant="destructive">
                <AlertDescription>{loadError}</AlertDescription>
              </Alert>
            ) : null}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
