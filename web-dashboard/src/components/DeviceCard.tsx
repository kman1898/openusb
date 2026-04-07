import type { UsbDevice, ClientStatus } from "../types";
import { StatusIndicator } from "./StatusIndicator";
import { useState } from "react";

interface Props {
  device: UsbDevice;
  client: ClientStatus;
  serverHost: string;
  onShare: (busId: string) => Promise<void>;
  onUnshare: (busId: string) => Promise<void>;
  onAttach: (server: string, busId: string) => Promise<void>;
  onDetach: (busId: string) => Promise<void>;
  onNickname: (busId: string, name: string) => Promise<void>;
}

export function DeviceCard({
  device,
  client,
  serverHost,
  onShare,
  onUnshare,
  onAttach,
  onDetach,
  onNickname,
}: Props) {
  const [loading, setLoading] = useState(false);
  const [editing, setEditing] = useState(false);
  const [nick, setNick] = useState(device.nickname || "");
  const [error, setError] = useState<string | null>(null);

  const act = async (fn: () => Promise<void>) => {
    setLoading(true);
    setError(null);
    try {
      await fn();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed");
    } finally {
      setLoading(false);
    }
  };

  const vidPid = `${device.vendor_id.toString(16).padStart(4, "0")}:${device.product_id.toString(16).padStart(4, "0")}`;
  const displayName = device.nickname || device.product_name || "Unknown Device";

  return (
    <div
      style={{
        padding: 16,
        backgroundColor: "#1e1e2e",
        borderRadius: 8,
        border: "1px solid #313244",
      }}
    >
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
        <div>
          {editing ? (
            <form
              onSubmit={(e) => {
                e.preventDefault();
                onNickname(device.bus_id, nick).then(() => setEditing(false));
              }}
              style={{ display: "flex", gap: 4 }}
            >
              <input
                value={nick}
                onChange={(e) => setNick(e.target.value)}
                style={{
                  background: "#11111b",
                  border: "1px solid #45475a",
                  borderRadius: 4,
                  padding: "2px 6px",
                  color: "#cdd6f4",
                  fontSize: 14,
                }}
                autoFocus
              />
              <button type="submit" style={btnStyle}>Save</button>
              <button type="button" onClick={() => setEditing(false)} style={btnStyle}>Cancel</button>
            </form>
          ) : (
            <h3
              style={{ margin: 0, color: "#cdd6f4", cursor: "pointer", fontSize: 16 }}
              onClick={() => setEditing(true)}
              title="Click to rename"
            >
              {displayName}
            </h3>
          )}
          <div style={{ color: "#a6adc8", fontSize: 12, marginTop: 4 }}>
            {device.vendor_name && <span>{device.vendor_name} &middot; </span>}
            <span>{vidPid}</span> &middot; <span>Bus {device.bus_id}</span>
            {device.serial && <span> &middot; S/N {device.serial}</span>}
          </div>
          <div style={{ color: "#a6adc8", fontSize: 12, marginTop: 2 }}>
            Speed: {device.speed}
          </div>
        </div>
        <StatusIndicator state={device.state} />
      </div>

      {device.state.status === "in_use" && (
        <div style={{ color: "#89b4fa", fontSize: 12, marginTop: 8 }}>
          In use by {device.state.client_name || device.state.client_ip}
        </div>
      )}

      <div style={{ display: "flex", gap: 8, marginTop: 12 }}>
        {device.state.status === "not_shared" && (
          <button style={btnPrimary} disabled={loading} onClick={() => act(() => onShare(device.bus_id))}>
            Share
          </button>
        )}
        {device.state.status === "available" && (
          <>
            <button style={btnDanger} disabled={loading} onClick={() => act(() => onUnshare(device.bus_id))}>
              Unshare
            </button>
            {client.installed && (
              <button style={btnPrimary} disabled={loading} onClick={() => act(() => onAttach(serverHost, device.bus_id))}>
                Use on this PC
              </button>
            )}
          </>
        )}
        {device.state.status === "in_use" && client.installed && (
          <button style={btnDanger} disabled={loading} onClick={() => act(() => onDetach(device.bus_id))}>
            Release
          </button>
        )}
      </div>

      {error && <div style={{ color: "#f38ba8", fontSize: 12, marginTop: 6 }}>{error}</div>}
    </div>
  );
}

const btnStyle: React.CSSProperties = {
  padding: "4px 10px",
  borderRadius: 4,
  border: "1px solid #45475a",
  background: "#313244",
  color: "#cdd6f4",
  cursor: "pointer",
  fontSize: 12,
};

const btnPrimary: React.CSSProperties = {
  ...btnStyle,
  background: "#1e66f5",
  border: "1px solid #1e66f5",
  color: "#fff",
};

const btnDanger: React.CSSProperties = {
  ...btnStyle,
  background: "#d20f39",
  border: "1px solid #d20f39",
  color: "#fff",
};
