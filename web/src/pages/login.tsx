import { useState } from "react";
import { useLogin } from "@refinedev/core";
import { Navigate } from "react-router-dom";
import { getTokens } from "@/lib/auth-store";
import { useI18n } from "@/lib/i18n";
import { ViewControls } from "@/components/common/view-controls";
import { LoginForm } from "@/components/login-form";

export function LoginPage(): JSX.Element {
  const { t } = useI18n();
  const tokens = getTokens();
  const { mutate: login, isLoading } = useLogin();
  const [error, setError] = useState<string | null>(null);

  if (tokens) {
    return <Navigate to="/app/dashboard" replace />;
  }

  return (
    <div className="relative flex min-h-screen w-full items-center justify-center p-6 md:p-10">
      <div className="absolute right-6 top-6 z-10">
        <ViewControls compact />
      </div>
      <div className="w-full max-w-5xl">
        <LoginForm
          loading={isLoading}
          error={error}
          onLoginSubmit={({ username, password }) => {
            setError(null);
            login(
              { username, password },
              {
                onError: (e) => setError(e?.message ?? t("Login failed")),
              },
            );
          }}
        />
      </div>
    </div>
  );
}
