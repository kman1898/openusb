import type { DriverStatus as DriverStatusType } from "../types";
import { useState } from "react";

interface Props {
  driver?: DriverStatusType;
  onInstall: () => Promise<void>;
}

export function DriverStatus({ driver, onInstall }: Props) {
  const [installing, setInstalling] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!driver) return null;

  const handleInstall = async () => {
    setInstalling(true);
    setError(null);
    try {
      await onInstall();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Install failed");
    } finally {
      setInstalling(false);
    }
  };

  if (driver.status === "installed") {
    return null;
  }

  return (
    <div
      style={{
        padding: "10px 16px",
        backgroundColor: "#331a1a",
        borderRadius: 8,
        border: "1px solid #7f1d1d",
        fontSize: 13,
        marginBottom: 12,
      }}
    >
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <div>
          <strong style={{ color: "#f87171" }}>
            {driver.status === "not_installed"
              ? "USB/IP driver not installed"
              : `Driver error: ${driver.message}`}
          </strong>
          <div style={{ color: "#fca5a5", marginTop: 2 }}>
            The USB/IP driver is required to attach remote USB devices to this machine.
          </div>
        </div>
        <button
          onClick={handleInstall}
          disabled={installing}
          style={{
            padding: "6px 12px",
            borderRadius: 4,
            border: "1px solid #dc2626",
            background: "#dc2626",
            color: "#fff",
            cursor: "pointer",
            fontSize: 12,
            whiteSpace: "nowrap",
          }}
        >
          {installing ? "Installing..." : "Install Driver"}
        </button>
      </div>
      {error && <div style={{ color: "#f38ba8", fontSize: 12, marginTop: 6 }}>{error}</div>}
    </div>
  );
}
