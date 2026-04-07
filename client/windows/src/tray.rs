// System tray icon for Windows.
//
// Tray icon states:
// - Green: connected to server(s), device(s) in use
// - Yellow: connected to server(s), no devices in use
// - Gray: no servers found
// - Red: error or driver issue
//
// Right-click menu:
// - Open Dashboard → launches browser to server URL
// - Status: "N devices connected"
// - Auto-start on login (toggle)
// - About
// - Exit
//
// Implementation deferred to Windows-specific build.
// The tray module will use the `windows` crate for Win32 Shell_NotifyIcon APIs.
