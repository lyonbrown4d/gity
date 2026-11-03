import { useState } from "react";
import { useLogin } from "@refinedev/core";
import { Navigate, useNavigate } from "react-router";
import { getTokens } from "@/lib/auth-store";
import { useI18n } from "@/lib/i18n";
import { ViewControls } from "@/components/common/view-controls";
import { LoginForm } from "@/components/login-form";

export const LoginPage = (): JSX.Element => {
  const { t } = useI18n();
  const navigate = useNavigate();
  const tokens = getTokens();
  const { mutate: login, isPending: isLoading } = useLogin();
  const [error, setError] = useState<string | null>(null);

  if (tokens) {
    return <Navigate to="/app/dashboard" replace />;
  }

  return (
    <main className="relative flex min-h-screen w-full items-center justify-center overflow-hidden bg-background p-4 sm:p-6 md:p-10">
      <div className="absolute inset-0 gity-soft-grid opacity-30" aria-hidden="true" />
      <div className="absolute inset-x-0 top-0 h-px bg-primary/60" aria-hidden="true" />
      <div className="absolute right-4 top-4 z-10 sm:right-6 sm:top-6">
        <ViewControls compact />
      </div>
      <div className="relative w-full max-w-4xl">
        <LoginForm
          loading={isLoading}
          error={error}
          onLoginSubmit={({ username, password }) => {
            setError(null);
            login(
              { username, password },
              {
                onError: (e) => setError(e?.message ?? t("Login failed")),
                onSuccess: ({ success }) => {
                  if (success) {
                    navigate("/app/dashboard", { replace: true });
                  }
                },
              },
            );
          }}
        />
      </div>
    </main>
  );
};
