import { useEffect, useRef, useCallback, useState } from "react";
import type { ServerEvent, UsbDevice } from "../types";

export function useWebSocket(onEvent: (event: ServerEvent) => void) {
  const wsRef = useRef<WebSocket | null>(null);
  const mountedRef = useRef(true);
  const [connected, setConnected] = useState(false);
  const onEventRef = useRef(onEvent);
  onEventRef.current = onEvent;

  const connect = useCallback(() => {
    if (!mountedRef.current) return;
    if (wsRef.current?.readyState === WebSocket.OPEN || wsRef.current?.readyState === WebSocket.CONNECTING) return;

    const proto = window.location.protocol === "https:" ? "wss:" : "ws:";
    const token = localStorage.getItem("openusb_token");
    const tokenParam = token ? `?token=${encodeURIComponent(token)}` : "";
    const url = `${proto}//${window.location.host}/api/v1/events${tokenParam}`;

    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onopen = () => {
      if (mountedRef.current) setConnected(true);
    };

    ws.onmessage = (msg) => {
      try {
        const data = JSON.parse(msg.data);
        if (Array.isArray(data)) {
          for (const device of data as UsbDevice[]) {
            onEventRef.current({ type: "device_attached", device });
          }
        } else {
          onEventRef.current(data as ServerEvent);
        }
      } catch {
        // ignore malformed messages
      }
    };

    ws.onclose = () => {
      if (mountedRef.current) {
        setConnected(false);
        setTimeout(connect, 3000);
      }
    };

    ws.onerror = () => ws.close();
  }, []);

  useEffect(() => {
    mountedRef.current = true;
    connect();
    return () => {
      mountedRef.current = false;
      wsRef.current?.close();
    };
  }, [connect]);

  return { connected };
}
