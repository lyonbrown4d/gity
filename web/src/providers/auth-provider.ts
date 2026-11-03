import type { AuthProvider } from "@refinedev/core";
import { clearTokens, getTokens, setTokens } from "@/lib/auth-store";
import { generatedRequest } from "./refine/generated-dispatch";
import type { User } from "@/lib/generated/model/user";
import type { Session } from "@/lib/generated/model/session";
import { translateForCurrentLocale } from "@/lib/i18n";

export const authProvider: AuthProvider = {
  login: async ({ username, password }) => {
    try {
      const session = await generatedRequest<Session>("/auth/login", {
        method: "POST",
        payload: { username, password },
        auth: false,
        skipRefresh: true,
      });
      setTokens({ accessToken: session.token, refreshToken: session.refresh_token });
      return {
        success: true,
        redirectTo: "/app/dashboard",
      };
    } catch (error) {
      return {
        success: false,
        error: {
          name: "LoginError",
          message: error instanceof Error ? error.message : translateForCurrentLocale("Login failed"),
        },
      };
    }
  },
  logout: async () => {
    const tokens = getTokens();
    try {
      if (tokens)
        await generatedRequest("/auth/logout", { method: "POST", payload: { refresh_token: tokens.refreshToken } });
    } finally {
      clearTokens();
    }
    return {
      success: true,
      redirectTo: "/login",
    };
  },
  check: async () => {
    const tokens = getTokens();
    if (!tokens) {
      return {
        authenticated: false,
        logout: true,
        redirectTo: "/login",
      };
    }
    try {
      await generatedRequest<User>("/users/me");
      return {
        authenticated: true,
      };
    } catch {
      clearTokens();
      return {
        authenticated: false,
        logout: true,
        redirectTo: "/login",
      };
    }
  },
  getPermissions: async () => {
    try {
      const user = await generatedRequest<User>("/users/me");
      return {
        isSuperAdmin: user.is_super_admin,
      };
    } catch {
      return null;
    }
  },
  getIdentity: async () => {
    try {
      const user = await generatedRequest<User>("/users/me");
      return {
        id: user.id,
        name: user.username,
        email: user.email,
        isSuperAdmin: user.is_super_admin,
      };
    } catch {
      return null;
    }
  },
  onError: async (error) => {
    if (error?.statusCode === 401) {
      clearTokens();
      return {
        logout: true,
        redirectTo: "/login",
      };
    }
    return { error };
  },
};
