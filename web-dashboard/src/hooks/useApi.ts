import type { ServerInfo, UsbDevice } from "../types";

const API_BASE = `${window.location.origin}/api/v1`;

function getToken(): string | null {
  return localStorage.getItem("openusb_token");
}

async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const token = getToken();
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(init?.headers as Record<string, string> || {}),
  };
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  const { headers: _dropped, ...restInit } = init || {};

  const res = await fetch(`${API_BASE}${path}`, {
    headers,
    ...restInit,
  });
  if (res.status === 401) {
    localStorage.removeItem("openusb_token");
    localStorage.removeItem("openusb_username");
    localStorage.removeItem("openusb_role");
    window.location.reload();
    throw new Error("Session expired");
  }
  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(body.error || res.statusText);
  }
  if (res.status === 204 || res.headers.get("content-length") === "0") {
    return undefined as T;
  }
  return res.json();
}

export function useApi() {
  return {
    getServerInfo: () => apiFetch<ServerInfo>("/server/info"),
    getDevices: () => apiFetch<UsbDevice[]>("/devices"),
    shareDevice: (busId: string) =>
      apiFetch<void>(`/devices/${encodeURIComponent(busId)}/share`, { method: "POST" }),
    unshareDevice: (busId: string) =>
      apiFetch<void>(`/devices/${encodeURIComponent(busId)}/unshare`, { method: "POST" }),
    setNickname: (busId: string, nickname: string) =>
      apiFetch<void>(`/devices/${encodeURIComponent(busId)}/nickname`, {
        method: "PUT",
        body: JSON.stringify({ nickname }),
      }),
  };
}
