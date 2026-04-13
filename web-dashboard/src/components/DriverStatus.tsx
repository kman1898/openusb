import type { DriverStatus as DriverStatusType } from "../types";

interface Props {
  driver?: DriverStatusType;
  os?: string;
}

export function DriverStatus({ driver, os }: Props) {
  if (!driver) return null;
  if (driver.status === "installed") return null;

  // macOS doesn't support USB/IP device attachment — only management
  if (os === "macos") return null;

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
      <strong style={{ color: "#f87171" }}>
        {driver.status === "not_installed"
          ? "USB/IP driver not installed"
          : `Driver error: ${driver.message}`}
      </strong>
      <div style={{ color: "#fca5a5", marginTop: 2 }}>
        The USB/IP driver is required to attach remote USB devices to this machine.
        {os === "windows" ? (
          <> Install the <code style={{ color: "#f87171" }}>usbip-win2</code> driver.</>
        ) : (
          <> Install with: <code style={{ color: "#f87171" }}>sudo apt install linux-tools-generic</code> (Ubuntu/Debian)
          or <code style={{ color: "#f87171" }}>sudo dnf install usbip-utils</code> (Fedora/RHEL).</>
        )}
      </div>
    </div>
  );
}
