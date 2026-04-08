import { useState } from "react";

interface Props {
  onLogin: (username: string, password: string) => Promise<void>;
}

export function LoginPage({ onLogin }: Props) {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      await onLogin(username, password);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Login failed");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ display: "flex", justifyContent: "center", alignItems: "center", minHeight: "100vh" }}>
      <div style={{ width: 360, padding: 32, backgroundColor: "#1e1e2e", borderRadius: 12, border: "1px solid #313244" }}>
        <h1 style={{ color: "#cdd6f4", fontSize: 22, marginBottom: 4, textAlign: "center" }}>OpenUSB</h1>
        <p style={{ color: "#a6adc8", fontSize: 13, marginBottom: 24, textAlign: "center" }}>Sign in to manage devices</p>
        <form onSubmit={handleSubmit}>
          <div style={{ marginBottom: 12 }}>
            <label style={{ color: "#a6adc8", fontSize: 12, display: "block", marginBottom: 4 }}>Username</label>
            <input
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              style={inputStyle}
              autoFocus
              required
            />
          </div>
          <div style={{ marginBottom: 16 }}>
            <label style={{ color: "#a6adc8", fontSize: 12, display: "block", marginBottom: 4 }}>Password</label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              style={inputStyle}
              required
            />
          </div>
          {error && <div style={{ color: "#f38ba8", fontSize: 12, marginBottom: 12 }}>{error}</div>}
          <button type="submit" disabled={loading} style={btnStyle}>
            {loading ? "Signing in..." : "Sign In"}
          </button>
        </form>
      </div>
    </div>
  );
}

const inputStyle: React.CSSProperties = {
  width: "100%",
  padding: "8px 10px",
  background: "#11111b",
  border: "1px solid #45475a",
  borderRadius: 6,
  color: "#cdd6f4",
  fontSize: 14,
  boxSizing: "border-box",
};

const btnStyle: React.CSSProperties = {
  width: "100%",
  padding: "10px",
  borderRadius: 6,
  border: "none",
  background: "#1e66f5",
  color: "#fff",
  fontSize: 14,
  cursor: "pointer",
  fontWeight: 500,
};
