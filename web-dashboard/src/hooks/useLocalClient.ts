import { useState, useEffect, useCallback } from "react";
import type { ClientStatus, AutoUseRule } from "../types";

const CLIENT_API = "http://localhost:9245/api";

export function useLocalClient() {
  const [client, setClient] = useState<ClientStatus>({ installed: false });

  const detect = useCallback(async () => {
    try {
      const res = await fetch(`${CLIENT_API}/status`, { signal: AbortSignal.timeout(2000) });
      const data = await res.json();
      setClient({
        installed: true,
        version: data.version,
        driver_status: data.driver_status,
      });
    } catch {
      setClient({ installed: false });
    }
  }, []);

  useEffect(() => {
    detect();
    const interval = setInterval(detect, 10000);
    return () => clearInterval(interval);
  }, [detect]);

  const attach = useCallback(async (server: string, busId: string) => {
    const res = await fetch(`${CLIENT_API}/attach`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ server, busid: busId }),
    });
    if (!res.ok) throw new Error("Attach failed");
  }, []);

  const detach = useCallback(async (busId: string) => {
    const res = await fetch(`${CLIENT_API}/detach`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ busid: busId }),
    });
    if (!res.ok) throw new Error("Detach failed");
  }, []);

  const getAutoUseRules = useCallback(async (): Promise<AutoUseRule[]> => {
    try {
      const res = await fetch(`${CLIENT_API}/auto-use`);
      if (!res.ok) return [];
      return res.json();
    } catch {
      return [];
    }
  }, []);

  const addAutoUseRule = useCallback(async (rule: AutoUseRule) => {
    const res = await fetch(`${CLIENT_API}/auto-use`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(rule),
    });
    if (!res.ok) throw new Error("Failed to add auto-use rule");
  }, []);

  return { client, attach, detach, detect, getAutoUseRules, addAutoUseRule };
}
