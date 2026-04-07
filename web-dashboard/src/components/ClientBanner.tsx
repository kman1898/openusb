import type { ClientStatus } from "../types";

interface Props {
  client: ClientStatus;
}

export function ClientBanner({ client }: Props) {
  if (client.installed) {
    return (
      <div
        style={{
          padding: "8px 16px",
          backgroundColor: "#052e16",
          borderRadius: 8,
          border: "1px solid #166534",
          display: "flex",
          alignItems: "center",
          gap: 8,
          fontSize: 13,
        }}
      >
        <span style={{ color: "#22c55e" }}>Client connected</span>
        {client.version && (
          <span style={{ color: "#86efac" }}>v{client.version}</span>
        )}
        {client.driver_status && (
          <span style={{ color: "#86efac" }}>Driver: {client.driver_status}</span>
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
