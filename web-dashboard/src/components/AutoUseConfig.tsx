import { useState, useEffect } from "react";
import type { AutoUseRule } from "../types";

interface Props {
  clientInstalled: boolean;
  getRules: () => Promise<AutoUseRule[]>;
  addRule: (rule: AutoUseRule) => Promise<void>;
}

export function AutoUseConfig({ clientInstalled, getRules, addRule }: Props) {
  const [rules, setRules] = useState<AutoUseRule[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [ruleType, setRuleType] = useState<AutoUseRule["type"]>("device");
  const [vendorId, setVendorId] = useState("");
  const [productId, setProductId] = useState("");
  const [server, setServer] = useState("");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (clientInstalled) {
      getRules().then(setRules).catch(() => {});
    }
  }, [clientInstalled, getRules]);

  if (!clientInstalled) return null;

  const handleAdd = async () => {
    setError(null);
    let rule: AutoUseRule;
    switch (ruleType) {
      case "device":
        if (!vendorId || !productId) { setError("Vendor and product ID required"); return; }
        rule = { type: "device", vendor_id: vendorId, product_id: productId };
        break;
      case "vendor_id":
        if (!vendorId) { setError("Vendor ID required"); return; }
        rule = { type: "vendor_id", vendor_id: vendorId };
        break;
      case "server":
        if (!server) { setError("Server name required"); return; }
        rule = { type: "server", server };
        break;
      case "all":
        rule = { type: "all" };
        break;
    }
    try {
      await addRule(rule);
      setRules([...rules, rule]);
      setShowForm(false);
      setVendorId("");
      setProductId("");
      setServer("");
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed");
    }
  };

  const ruleLabel = (rule: AutoUseRule) => {
    switch (rule.type) {
      case "all": return "All devices on all servers";
      case "server": return `All devices on ${rule.server}`;
      case "vendor_id": return `Vendor ${rule.vendor_id}`;
      case "device": return `Device ${rule.vendor_id}:${rule.product_id}`;
    }
  };

  return (
    <div style={{ marginTop: 24 }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 12 }}>
        <h2 style={{ color: "#cdd6f4", fontSize: 18, margin: 0 }}>Auto-Use Rules</h2>
        <button style={btnStyle} onClick={() => setShowForm(!showForm)}>
          {showForm ? "Cancel" : "Add Rule"}
        </button>
      </div>

      {showForm && (
        <div style={{ padding: 12, backgroundColor: "#1e1e2e", borderRadius: 8, border: "1px solid #313244", marginBottom: 12 }}>
          <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
            <select
              value={ruleType}
              onChange={(e) => setRuleType(e.target.value as AutoUseRule["type"])}
              style={selectStyle}
            >
              <option value="device">Specific Device</option>
              <option value="vendor_id">Vendor ID</option>
              <option value="server">Server</option>
              <option value="all">All Devices</option>
            </select>
            {(ruleType === "device" || ruleType === "vendor_id") && (
              <input
                placeholder="Vendor ID (e.g. 0765)"
                value={vendorId}
                onChange={(e) => setVendorId(e.target.value)}
                style={inputStyle}
              />
            )}
            {ruleType === "device" && (
              <input
                placeholder="Product ID (e.g. 5020)"
                value={productId}
                onChange={(e) => setProductId(e.target.value)}
                style={inputStyle}
              />
            )}
            {ruleType === "server" && (
              <input
                placeholder="Server name"
                value={server}
                onChange={(e) => setServer(e.target.value)}
                style={inputStyle}
              />
            )}
            <button style={btnPrimary} onClick={handleAdd}>Add</button>
          </div>
          {error && <div style={{ color: "#f38ba8", fontSize: 12, marginTop: 6 }}>{error}</div>}
        </div>
      )}

      {rules.length === 0 ? (
        <div style={{ color: "#6c7086", fontSize: 13 }}>
          No auto-use rules configured. Devices must be attached manually.
        </div>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
          {rules.map((rule, i) => (
            <div key={i} style={{ padding: "8px 12px", backgroundColor: "#1e1e2e", borderRadius: 6, border: "1px solid #313244", fontSize: 13, color: "#cdd6f4" }}>
              {ruleLabel(rule)}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

const inputStyle: React.CSSProperties = {
  background: "#11111b",
  border: "1px solid #45475a",
  borderRadius: 4,
  padding: "4px 8px",
  color: "#cdd6f4",
  fontSize: 13,
  width: 140,
};

const selectStyle: React.CSSProperties = {
  ...inputStyle,
  width: "auto",
};

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
