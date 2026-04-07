// macOS menu bar icon for OpenUSB.
//
// Menu bar icon states:
// - Green: connected to server(s), device(s) in use
// - Yellow: connected to server(s), no devices in use
// - Gray: no servers found
// - Red: error or driver issue
//
// Click menu:
// - Open Dashboard → launches browser to server URL
// - Status: "N devices connected"
// - Auto-start on login (toggle)
// - About
// - Quit
//
// Implementation will use a lightweight macOS menu bar framework.
// For now, the client runs headless — all UI is in the web dashboard.
