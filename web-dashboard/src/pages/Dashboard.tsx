import { useState, useEffect, useCallback } from "react";
import type { UsbDevice, ServerInfo, ServerEvent } from "../types";
import { useApi } from "../hooks/useApi";
import { useWebSocket } from "../hooks/useWebSocket";
import { useLocalClient } from "../hooks/useLocalClient";
import { DeviceCard } from "../components/DeviceCard";
import { ClientBanner } from "../components/ClientBanner";
import { AutoUseConfig } from "../components/AutoUseConfig";
import { DriverStatus } from "../components/DriverStatus";

export function Dashboard() {
  const api = useApi();
  const { client, attach, detach, getAutoUseRules, addAutoUseRule } = useLocalClient();
  const [devices, setDevices] = useState<Map<string, UsbDevice>>(new Map());
  const [serverInfo, setServerInfo] = useState<ServerInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api.getServerInfo().then(setServerInfo).catch((e) => setError(e.message));
  }, []);

  useEffect(() => {
    api
      .getDevices()
      .then((list) => {
        const map = new Map<string, UsbDevice>();
        for (const d of list) map.set(d.bus_id, d);
        setDevices(map);
      })
      .catch((e) => setError(e.message));
  }, []);

  const handleEvent = useCallback((event: ServerEvent) => {
    setDevices((prev) => {
      const next = new Map(prev);
      switch (event.type) {
        case "device_attached":
          next.set(event.device.bus_id, event.device);
          break;
        case "device_detached":
          next.delete(event.bus_id);
          break;
        case "device_shared": {
          const d = next.get(event.bus_id);
          if (d) next.set(event.bus_id, { ...d, state: { status: "available" } });
          break;
        }
        case "device_unshared": {
          const d = next.get(event.bus_id);
          if (d) next.set(event.bus_id, { ...d, state: { status: "not_shared" } });
          break;
        }
        case "device_in_use": {
          const d = next.get(event.bus_id);
          if (d)
            next.set(event.bus_id, {
              ...d,
              state: {
                status: "in_use",
                client_ip: event.client_ip,
                client_name: null,
                since: new Date().toISOString(),
              },
            });
          break;
        }
        case "device_released": {
          const d = next.get(event.bus_id);
          if (d) next.set(event.bus_id, { ...d, state: { status: "available" } });
          break;
        }
      }
      return next;
    });
  }, []);

  const { connected } = useWebSocket(handleEvent);

  const serverHost = window.location.hostname;
  const deviceList = Array.from(devices.values());
  const sharedCount = deviceList.filter((d) => d.state.status !== "not_shared").length;
  const inUseCount = deviceList.filter((d) => d.state.status === "in_use").length;

  const formatUptime = (s: number) => {
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    return h > 0 ? `${h}h ${m}m` : `${m}m`;
  };

  return (
    <div style={{ maxWidth: 960, margin: "0 auto", padding: 24 }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 24 }}>
        <div>
          <h1 style={{ margin: 0, color: "#cdd6f4", fontSize: 24 }}>
            {serverInfo?.name || "OpenUSB"}
          </h1>
          {serverInfo && (
            <div style={{ color: "#a6adc8", fontSize: 13, marginTop: 4 }}>
              {serverInfo.hostname} &middot; v{serverInfo.version} &middot; Up{" "}
              {formatUptime(serverInfo.uptime_seconds)}
            </div>
          )}
        </div>
        <span
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: 6,
            color: connected ? "#22c55e" : "#f38ba8",
            fontSize: 13,
          }}
        >
          <span
            style={{
              width: 8,
              height: 8,
              borderRadius: "50%",
              backgroundColor: connected ? "#22c55e" : "#f38ba8",
            }}
          />
          {connected ? "Live" : "Reconnecting..."}
        </span>
      </div>

      <div style={{ marginBottom: 16 }}>
        <ClientBanner client={client} />
      </div>

      {client.installed && client.driver_status && client.driver_status.status !== "installed" && (
        <DriverStatus driver={client.driver_status} />
      )}

      <div style={{ display: "flex", gap: 16, marginBottom: 24 }}>
        <StatCard label="Devices" value={deviceList.length} />
        <StatCard label="Shared" value={sharedCount} />
        <StatCard label="In Use" value={inUseCount} />
        <StatCard label="Clients" value={serverInfo?.client_count ?? 0} />
      </div>

      {error && (
        <div style={{ color: "#f38ba8", marginBottom: 16, fontSize: 13 }}>{error}</div>
      )}

      <h2 style={{ color: "#cdd6f4", fontSize: 18, marginBottom: 12 }}>USB Devices</h2>
      {deviceList.length === 0 ? (
        <div style={{ color: "#6c7086", padding: 32, textAlign: "center" }}>
          No USB devices detected
        </div>
      ) : (
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(400px, 1fr))", gap: 12 }}>
          {deviceList.map((device) => (
            <DeviceCard
              key={device.bus_id}
              device={device}
              client={client}
              serverHost={serverHost}
              onShare={api.shareDevice}
              onUnshare={api.unshareDevice}
              onAttach={attach}
              onDetach={detach}
              onNickname={api.setNickname}
            />
          ))}
        </div>
      )}

      <AutoUseConfig
        clientInstalled={client.installed}
        getRules={getAutoUseRules}
        addRule={addAutoUseRule}
      />
    </div>
  );
}

function StatCard({ label, value }: { label: string; value: number }) {
  return (
    <div
      style={{
        flex: 1,
        padding: "12px 16px",
        backgroundColor: "#1e1e2e",
        borderRadius: 8,
        border: "1px solid #313244",
        textAlign: "center",
      }}
    >
      <div style={{ color: "#cdd6f4", fontSize: 24, fontWeight: 600 }}>{value}</div>
      <div style={{ color: "#a6adc8", fontSize: 12 }}>{label}</div>
    </div>
  );
}
