import { useState, useEffect } from "react";

interface HistoryEntry {
  id: number;
  timestamp: string;
  event_type: string;
  bus_id: string | null;
  device_name: string | null;
  client_ip: string | null;
  username: string | null;
  details: string | null;
}

interface BandwidthStat {
  bus_id: string;
  bytes_sent: number;
  bytes_received: number;
  last_updated: string;
}

interface LatencyStat {
  bus_id: string;
  rtt_us: number;
  avg_rtt_us: number;
  samples: number;
  last_measured: string;
}

interface Props {
  authFetch: (path: string, init?: RequestInit) => Promise<Response>;
}

export function LogsPage({ authFetch }: Props) {
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [bandwidth, setBandwidth] = useState<BandwidthStat[]>([]);
  const [latency, setLatency] = useState<LatencyStat[]>([]);
  const [filter, setFilter] = useState("");
  const [tab, setTab] = useState<"events" | "bandwidth" | "latency">("events");

  const fetchData = async () => {
    try {
      const url = filter ? `/history?limit=200&event_type=${filter}` : "/history?limit=200";
      const res = await authFetch(url);
      if (res.ok) setHistory(await res.json());
    } catch { /* ignore */ }
    try {
      const res = await authFetch("/metrics/bandwidth");
      if (res.ok) setBandwidth(await res.json());
    } catch { /* ignore */ }
    try {
      const res = await authFetch("/metrics/latency");
      if (res.ok) setLatency(await res.json());
    } catch { /* ignore */ }
  };

  useEffect(() => { fetchData(); }, [filter]);
  useEffect(() => {
    const interval = setInterval(fetchData, 5000);
    return () => clearInterval(interval);
  }, [filter]);

  const formatBytes = (b: number) => {
    if (b < 1024) return `${b} B`;
    if (b < 1024 * 1024) return `${(b / 1024).toFixed(1)} KB`;
    return `${(b / 1024 / 1024).toFixed(1)} MB`;
  };

  const eventTypes = [
    "", "device_attached", "device_detached", "device_shared", "device_unshared",
    "client_connected", "client_disconnected", "device_in_use", "device_released",
    "auth_failed", "bandwidth_alert",
  ];

  const eventColor: Record<string, string> = {
    device_attached: "#22c55e",
    device_detached: "#f38ba8",
    device_shared: "#89b4fa",
    device_unshared: "#fab387",
    client_connected: "#22c55e",
    client_disconnected: "#f38ba8",
    device_in_use: "#3b82f6",
    device_released: "#a6adc8",
    auth_failed: "#f38ba8",
    bandwidth_alert: "#fbbf24",
  };

  return (
    <div>
      <h2 style={{ color: "#cdd6f4", fontSize: 18, marginBottom: 16 }}>Logs & Metrics</h2>

      <div style={{ display: "flex", gap: 8, marginBottom: 16 }}>
        {(["events", "bandwidth", "latency"] as const).map((t) => (
          <button key={t} onClick={() => setTab(t)} style={tab === t ? tabActive : tabBtn}>
            {t.charAt(0).toUpperCase() + t.slice(1)}
          </button>
        ))}
      </div>

      {tab === "events" && (
        <>
          <div style={{ marginBottom: 12 }}>
            <select value={filter} onChange={(e) => setFilter(e.target.value)} style={selectStyle}>
              <option value="">All events</option>
              {eventTypes.filter(Boolean).map((t) => (
                <option key={t} value={t}>{t.replace(/_/g, " ")}</option>
              ))}
            </select>
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
            {history.length === 0 ? (
              <div style={{ color: "#6c7086", padding: 24, textAlign: "center" }}>No events recorded yet</div>
            ) : (
              history.map((entry) => (
                <div key={entry.id} style={{ padding: "6px 10px", backgroundColor: "#1e1e2e", borderRadius: 4, border: "1px solid #313244", fontSize: 12, display: "flex", gap: 12, alignItems: "center" }}>
                  <span style={{ color: "#6c7086", minWidth: 140 }}>{entry.timestamp}</span>
                  <span style={{ color: eventColor[entry.event_type] || "#a6adc8", minWidth: 140 }}>
                    {entry.event_type.replace(/_/g, " ")}
                  </span>
                  {entry.bus_id && <span style={{ color: "#cdd6f4" }}>Bus: {entry.bus_id}</span>}
                  {entry.device_name && <span style={{ color: "#a6adc8" }}>{entry.device_name}</span>}
                  {entry.client_ip && <span style={{ color: "#89b4fa" }}>{entry.client_ip}</span>}
                  {entry.details && <span style={{ color: "#6c7086" }}>{entry.details}</span>}
                </div>
              ))
            )}
          </div>
        </>
      )}

      {tab === "bandwidth" && (
        <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          {bandwidth.length === 0 ? (
            <div style={{ color: "#6c7086", padding: 24, textAlign: "center" }}>No bandwidth data yet</div>
          ) : (
            bandwidth.map((stat) => (
              <div key={stat.bus_id} style={{ padding: "10px 14px", backgroundColor: "#1e1e2e", borderRadius: 8, border: "1px solid #313244" }}>
                <div style={{ color: "#cdd6f4", fontWeight: 500, fontSize: 14 }}>Bus {stat.bus_id}</div>
                <div style={{ display: "flex", gap: 24, marginTop: 4, fontSize: 12, color: "#a6adc8" }}>
                  <span>Sent: {formatBytes(stat.bytes_sent)}</span>
                  <span>Received: {formatBytes(stat.bytes_received)}</span>
                  <span>Total: {formatBytes(stat.bytes_sent + stat.bytes_received)}</span>
                </div>
              </div>
            ))
          )}
        </div>
      )}

      {tab === "latency" && (
        <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          {latency.length === 0 ? (
            <div style={{ color: "#6c7086", padding: 24, textAlign: "center" }}>No latency data yet</div>
          ) : (
            latency.map((stat) => (
              <div key={stat.bus_id} style={{ padding: "10px 14px", backgroundColor: "#1e1e2e", borderRadius: 8, border: "1px solid #313244" }}>
                <div style={{ color: "#cdd6f4", fontWeight: 500, fontSize: 14 }}>Bus {stat.bus_id}</div>
                <div style={{ display: "flex", gap: 24, marginTop: 4, fontSize: 12, color: "#a6adc8" }}>
                  <span>Current: {(stat.rtt_us / 1000).toFixed(1)}ms</span>
                  <span>Average: {(stat.avg_rtt_us / 1000).toFixed(1)}ms</span>
                  <span>Samples: {stat.samples}</span>
                </div>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
}

const tabBtn: React.CSSProperties = {
  padding: "6px 14px",
  borderRadius: 4,
  border: "1px solid transparent",
  background: "transparent",
  color: "#a6adc8",
  cursor: "pointer",
  fontSize: 13,
};

const tabActive: React.CSSProperties = {
  ...tabBtn,
  color: "#cdd6f4",
  background: "#313244",
  border: "1px solid #45475a",
};

const selectStyle: React.CSSProperties = {
  background: "#11111b",
  border: "1px solid #45475a",
  borderRadius: 4,
  padding: "4px 8px",
  color: "#cdd6f4",
  fontSize: 13,
};
