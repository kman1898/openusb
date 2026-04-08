import { useState } from "react";
import { Dashboard } from "./pages/Dashboard";
import { LoginPage } from "./pages/LoginPage";
import { UsersPage } from "./pages/UsersPage";
import { LogsPage } from "./pages/LogsPage";
import { useAuth } from "./hooks/useAuth";

type Page = "dashboard" | "users" | "logs";

function App() {
  const { isAuthenticated, isAdmin, username, login, logout, authFetch } = useAuth();
  const [page, setPage] = useState<Page>("dashboard");

  if (!isAuthenticated) {
    return (
      <div style={rootStyle}>
        <LoginPage onLogin={login} />
      </div>
    );
  }

  return (
    <div style={rootStyle}>
      <nav style={navStyle}>
        <div style={{ display: "flex", gap: 12, alignItems: "center" }}>
          <span style={{ color: "#cdd6f4", fontWeight: 600, fontSize: 15 }}>OpenUSB</span>
          <button onClick={() => setPage("dashboard")} style={page === "dashboard" ? navBtnActive : navBtn}>
            Devices
          </button>
          <button onClick={() => setPage("logs")} style={page === "logs" ? navBtnActive : navBtn}>
            Logs
          </button>
          {isAdmin && (
            <button onClick={() => setPage("users")} style={page === "users" ? navBtnActive : navBtn}>
              Users & Access
            </button>
          )}
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <span style={{ color: "#a6adc8", fontSize: 12 }}>{username}</span>
          <button onClick={logout} style={navBtn}>Sign Out</button>
        </div>
      </nav>

      <main style={{ padding: "0 24px 24px" }}>
        {page === "dashboard" && <Dashboard />}
        {page === "logs" && <LogsPage authFetch={authFetch} />}
        {page === "users" && isAdmin && <UsersPage authFetch={authFetch} />}
      </main>
    </div>
  );
}

export default App;

const rootStyle: React.CSSProperties = {
  minHeight: "100vh",
  backgroundColor: "#11111b",
  color: "#cdd6f4",
  fontFamily: "system-ui, -apple-system, sans-serif",
};

const navStyle: React.CSSProperties = {
  display: "flex",
  justifyContent: "space-between",
  alignItems: "center",
  padding: "10px 24px",
  borderBottom: "1px solid #313244",
  marginBottom: 16,
};

const navBtn: React.CSSProperties = {
  padding: "4px 10px",
  borderRadius: 4,
  border: "1px solid transparent",
  background: "transparent",
  color: "#a6adc8",
  cursor: "pointer",
  fontSize: 13,
};

const navBtnActive: React.CSSProperties = {
  ...navBtn,
  color: "#cdd6f4",
  background: "#313244",
  border: "1px solid #45475a",
};
