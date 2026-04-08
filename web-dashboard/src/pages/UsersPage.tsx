import { useState, useEffect } from "react";

interface User {
  id: number;
  username: string;
  role: string;
  enabled: boolean;
  created_at: string;
}

interface Props {
  authFetch: (path: string, init?: RequestInit) => Promise<Response>;
}

export function UsersPage({ authFetch }: Props) {
  const [users, setUsers] = useState<User[]>([]);
  const [showCreate, setShowCreate] = useState(false);
  const [newUsername, setNewUsername] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [newRole, setNewRole] = useState("user");
  const [error, setError] = useState<string | null>(null);

  const fetchUsers = async () => {
    try {
      const res = await authFetch("/users");
      if (res.ok) setUsers(await res.json());
    } catch {
      // ignore
    }
  };

  useEffect(() => { fetchUsers(); }, []);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    try {
      const res = await authFetch("/users", {
        method: "POST",
        body: JSON.stringify({ username: newUsername, password: newPassword, role: newRole }),
      });
      if (!res.ok) {
        const body = await res.json();
        throw new Error(body.error);
      }
      setShowCreate(false);
      setNewUsername("");
      setNewPassword("");
      setNewRole("user");
      fetchUsers();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed");
    }
  };

  const toggleUser = async (username: string, enabled: boolean) => {
    await authFetch(`/users/${encodeURIComponent(username)}`, {
      method: "PUT",
      body: JSON.stringify({ enabled }),
    });
    fetchUsers();
  };

  const changeRole = async (username: string, role: string) => {
    await authFetch(`/users/${encodeURIComponent(username)}`, {
      method: "PUT",
      body: JSON.stringify({ role }),
    });
    fetchUsers();
  };

  const deleteUser = async (username: string) => {
    await authFetch(`/users/${encodeURIComponent(username)}`, { method: "DELETE" });
    fetchUsers();
  };

  return (
    <div>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
        <h2 style={{ color: "#cdd6f4", fontSize: 18, margin: 0 }}>Users & Access</h2>
        <button onClick={() => setShowCreate(!showCreate)} style={btnStyle}>
          {showCreate ? "Cancel" : "Add User"}
        </button>
      </div>

      {showCreate && (
        <form onSubmit={handleCreate} style={{ padding: 12, backgroundColor: "#1e1e2e", borderRadius: 8, border: "1px solid #313244", marginBottom: 16 }}>
          <div style={{ display: "flex", gap: 8, flexWrap: "wrap", alignItems: "end" }}>
            <div>
              <label style={labelStyle}>Username</label>
              <input value={newUsername} onChange={(e) => setNewUsername(e.target.value)} style={inputStyle} required />
            </div>
            <div>
              <label style={labelStyle}>Password</label>
              <input type="password" value={newPassword} onChange={(e) => setNewPassword(e.target.value)} style={inputStyle} required />
            </div>
            <div>
              <label style={labelStyle}>Role</label>
              <select value={newRole} onChange={(e) => setNewRole(e.target.value)} style={inputStyle}>
                <option value="admin">Admin</option>
                <option value="user">User</option>
                <option value="viewer">Viewer</option>
              </select>
            </div>
            <button type="submit" style={btnPrimary}>Create</button>
          </div>
          {error && <div style={{ color: "#f38ba8", fontSize: 12, marginTop: 6 }}>{error}</div>}
        </form>
      )}

      <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
        {users.map((user) => (
          <div key={user.id} style={{ padding: "10px 14px", backgroundColor: "#1e1e2e", borderRadius: 8, border: "1px solid #313244", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
            <div>
              <span style={{ color: "#cdd6f4", fontWeight: 500, fontSize: 14 }}>{user.username}</span>
              <span style={{ color: "#a6adc8", fontSize: 12, marginLeft: 8 }}>{user.role}</span>
              {!user.enabled && <span style={{ color: "#f38ba8", fontSize: 12, marginLeft: 8 }}>disabled</span>}
            </div>
            <div style={{ display: "flex", gap: 6 }}>
              <select
                value={user.role}
                onChange={(e) => changeRole(user.username, e.target.value)}
                style={{ ...inputStyle, width: "auto", padding: "2px 4px", fontSize: 11 }}
              >
                <option value="admin">Admin</option>
                <option value="user">User</option>
                <option value="viewer">Viewer</option>
              </select>
              <button onClick={() => toggleUser(user.username, !user.enabled)} style={btnSmall}>
                {user.enabled ? "Disable" : "Enable"}
              </button>
              <button onClick={() => deleteUser(user.username)} style={{ ...btnSmall, background: "#d20f39", border: "1px solid #d20f39" }}>
                Delete
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

const labelStyle: React.CSSProperties = { color: "#a6adc8", fontSize: 11, display: "block", marginBottom: 2 };
const inputStyle: React.CSSProperties = { background: "#11111b", border: "1px solid #45475a", borderRadius: 4, padding: "4px 8px", color: "#cdd6f4", fontSize: 13 };
const btnStyle: React.CSSProperties = { padding: "4px 10px", borderRadius: 4, border: "1px solid #45475a", background: "#313244", color: "#cdd6f4", cursor: "pointer", fontSize: 12 };
const btnPrimary: React.CSSProperties = { ...btnStyle, background: "#1e66f5", border: "1px solid #1e66f5", color: "#fff" };
const btnSmall: React.CSSProperties = { ...btnStyle, padding: "2px 8px", fontSize: 11 };
