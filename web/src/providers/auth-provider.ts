import type { AuthBindings } from "@refinedev/core";
import { clearTokens, getTokens } from "@/lib/auth-store";
import { apiRequest, login, logout } from "@/lib/api";
import { translateForCurrentLocale } from "@/lib/i18n";

interface CurrentUser {
  id: string;
  username: string;
  email: string;
  status: string;
  is_super_admin: boolean;
}

export const authProvider: AuthBindings = {
  login: async ({ username, password }) => {
    try {
      await login({ username, password });
      return {
        success: true,
        redirectTo: "/admin",
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
    await logout();
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
      await apiRequest<CurrentUser>("/users/me");
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
      const user = await apiRequest<CurrentUser>("/users/me");
      return {
        isSuperAdmin: user.is_super_admin,
      };
    } catch {
      return null;
    }
  },
  getIdentity: async () => {
    try {
      const user = await apiRequest<CurrentUser>("/users/me");
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
