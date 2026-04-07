import type { DeviceState } from "../types";

interface Props {
  state: DeviceState;
}

export function StatusIndicator({ state }: Props) {
  const status = state.status;
  const colors: Record<string, string> = {
    not_shared: "#888",
    available: "#22c55e",
    in_use: "#3b82f6",
  };
  const labels: Record<string, string> = {
    not_shared: "Not Shared",
    available: "Available",
    in_use: "In Use",
  };

  return (
    <span style={{ display: "inline-flex", alignItems: "center", gap: 6 }}>
      <span
        style={{
          width: 8,
          height: 8,
          borderRadius: "50%",
          backgroundColor: colors[status] || "#888",
        }}
      />
      <span style={{ fontSize: 13, color: colors[status] || "#888" }}>
        {labels[status] || status}
      </span>
    </span>
  );
}
