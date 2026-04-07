import type { ClientStatus } from "../types";

interface Props {
  client: ClientStatus;
}

export function ClientBanner({ client }: Props) {
  if (client.installed) {
    const driverText = client.driver_status
      ? client.driver_status.status === "installed"
        ? `Driver: ${client.driver_status.version}`
        : client.driver_status.status === "not_installed"
          ? "Driver: missing"
          : "Driver: error"
      : null;

    return (
      <div
        style={{
          padding: "8px 16px",
          backgroundColor: "#052e16",
          borderRadius: 8,
          border: "1px solid #166534",
          display: "flex",
          alignItems: "center",
          gap: 12,
          fontSize: 13,
        }}
      >
        <span style={{ color: "#22c55e", fontWeight: 500 }}>Client connected</span>
        {client.version && (
          <span style={{ color: "#86efac" }}>v{client.version}</span>
        )}
        {driverText && (
          <span style={{ color: "#86efac" }}>{driverText}</span>
        )}
      </div>
    );
  }

  return (
    <div
      style={{
        padding: "12px 16px",
        backgroundColor: "#422006",
        borderRadius: 8,
        border: "1px solid #92400e",
        fontSize: 13,
      }}
    >
      <strong style={{ color: "#fbbf24" }}>No local client detected.</strong>{" "}
      <span style={{ color: "#fde68a" }}>
        Install the OpenUSB client to attach devices to this machine.
        Attach/detach controls require the client running on localhost:9245.
      </span>
    </div>
  );
}
