import type { ServerInfo, UsbDevice } from "../types";

const API_BASE = `${window.location.origin}/api/v1`;

async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    headers: { "Content-Type": "application/json" },
    ...init,
  });
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
