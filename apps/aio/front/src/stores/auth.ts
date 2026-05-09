import { create } from "zustand";
import { getApiBaseUrl } from "@az/api-client";

interface AuthState {
  username: string | null;
  loading: boolean;
  error: string | null;
  checkSession: () => Promise<void>;
  login: (user: string, pass: string) => Promise<void>;
  logout: () => Promise<void>;
}

export const useAuthStore = create<AuthState>((set) => ({
  username: null,
  loading: true,
  error: null,

  checkSession: async () => {
    try {
      const baseUrl = getApiBaseUrl();
      const res = await fetch(`${baseUrl}/api/admin/session`, {
        credentials: "include",
      });
      const session = await res.json();
      set({
        username: session.authenticated ? session.username : null,
        loading: false,
        error: null,
      });
    } catch {
      set({ username: null, loading: false });
    }
  },

  login: async (username, password) => {
    const baseUrl = getApiBaseUrl();
    const res = await fetch(`${baseUrl}/api/admin/session/login`, {
      method: "POST",
      credentials: "include",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ username, password }),
    });
    if (!res.ok) {
      const text = await res.text();
      throw new Error(text || `HTTP ${res.status}`);
    }
    const session = await res.json();
    if (!session.authenticated) {
      throw new Error("登录失败");
    }
    set({ username: session.username, error: null });
  },

  logout: async () => {
    const baseUrl = getApiBaseUrl();
    await fetch(`${baseUrl}/api/admin/session/logout`, {
      method: "POST",
      credentials: "include",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({}),
    });
    set({ username: null });
  },
}));
