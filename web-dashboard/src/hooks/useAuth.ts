import { useState, useCallback } from "react";

const API_BASE = `${window.location.origin}/api/v1`;

interface AuthState {
  token: string | null;
  username: string | null;
  role: string | null;
}

export function useAuth() {
  const [auth, setAuth] = useState<AuthState>(() => {
    const token = localStorage.getItem("openusb_token");
    const username = localStorage.getItem("openusb_username");
    const role = localStorage.getItem("openusb_role");
    return { token, username, role };
  });

  const login = useCallback(async (username: string, password: string) => {
    const res = await fetch(`${API_BASE}/auth/login`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ username, password }),
    });
    if (!res.ok) {
      const body = await res.json().catch(() => ({ error: "Login failed" }));
      throw new Error(body.error || "Login failed");
    }
    const data = await res.json();
    localStorage.setItem("openusb_token", data.token);
    localStorage.setItem("openusb_username", data.username);
    localStorage.setItem("openusb_role", data.role);
    setAuth({ token: data.token, username: data.username, role: data.role });
  }, []);

  const logout = useCallback(() => {
    localStorage.removeItem("openusb_token");
    localStorage.removeItem("openusb_username");
    localStorage.removeItem("openusb_role");
    setAuth({ token: null, username: null, role: null });
  }, []);

  const authFetch = useCallback(
    async (path: string, init?: RequestInit) => {
      const headers: Record<string, string> = {
        "Content-Type": "application/json",
      };
      if (auth.token) {
        headers["Authorization"] = `Bearer ${auth.token}`;
      }
      const res = await fetch(`${API_BASE}${path}`, {
        ...init,
        headers: { ...headers, ...((init?.headers as Record<string, string>) || {}) },
      });
      if (res.status === 401) {
        logout();
        throw new Error("Session expired");
      }
      return res;
    },
    [auth.token, logout]
  );

  return {
    isAuthenticated: !!auth.token,
    username: auth.username,
    role: auth.role,
    isAdmin: auth.role === "admin",
    login,
    logout,
    authFetch,
  };
}
