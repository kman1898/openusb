# OpenUSB

**Share any USB device over your network — no limits, no licenses, fully open source.**

OpenUSB is an open-source USB over IP solution. It lets you share USB devices connected to one machine (like a Raspberry Pi) with any other machine on your network, as if the device were plugged in locally.

## Why OpenUSB?

Commercial USB-over-IP solutions charge per-server licenses and lock features behind paywalls. OpenUSB gives you everything for free:

- **Unlimited devices, servers, and clients** — always free
- **Auto-discovery** via mDNS/Bonjour — servers appear automatically
- **One-click connect** — Windows tray app with device tree
- **Auto-reconnect** — devices come back after network blips
- **TLS encryption + mutual certificate auth** — secure over any network
- **REST API + WebSocket** — full programmatic control
- **Web dashboard** — manage everything from a browser

## Quick Start

### Server (Raspberry Pi / Linux)

```bash
curl -fsSL https://get.openusb.dev | bash
```

The server shares all connected USB devices and advertises itself on the network.

### Client (CLI)

```bash
openusb list                         # Discover servers + devices
openusb use living-room-pi.1-1.3    # Attach a device
openusb stop living-room-pi.1-1.3   # Detach
```

## Project Status

OpenUSB is under active development. See the [roadmap](#roadmap) for what's coming.

| Component | Status |
|---|---|
| Server daemon (`openusbd`) | Scaffolded |
| Shared library | Scaffolded |
| CLI tool | Scaffolded |
| Client library | Scaffolded |
| Windows GUI client | Planned (Phase 2) |
| Web dashboard | Planned (Phase 4) |

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                     OPENUSB ECOSYSTEM                     │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  ┌─────────────┐    mDNS / API    ┌───────────────────┐ │
│  │  Pi Server  │◄────────────────►│  Client (CLI/GUI) │ │
│  │  (openusbd) │    USB/IP TCP    │                   │ │
│  │             │◄────────────────►│                   │ │
│  └──────┬──────┘                  └───────────────────┘ │
│         │ REST/WS                                        │
│         ▼                                                │
│  ┌──────────────┐                                        │
│  │ Web Dashboard │                                       │
│  └──────────────┘                                        │
└──────────────────────────────────────────────────────────┘
```

## Building from Source

Requires [Rust](https://rustup.rs/) 1.85+.

```bash
git clone https://github.com/kman1898/openusb.git
cd usb-passthrough
cargo build
```

## Roadmap

- **Phase 1** — Core server: USB enumeration, USB/IP binding, hotplug, mDNS, REST API
- **Phase 2** — Windows client: tray app, device tree, auto-connect, installer
- **Phase 3** — Security: TLS, mutual auth, ACLs, reverse connections
- **Phase 4** — Web dashboard: React UI, device management, logs, scheduling
- **Phase 5** — Distribution: Pi image, Docker, .deb/.rpm packages, auto-updater
- **Phase 6** — Beyond: mobile app, Home Assistant, MQTT, Tailscale integration

## Technology

| Component | Technology |
|---|---|
| Server | Rust (tokio, axum) |
| Client | Rust (clap, mdns-sd) |
| Dashboard | React + TypeScript + Vite |
| USB transport | Linux kernel USB/IP + usbip-win2 |
| TLS | rustls |
| Database | SQLite |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## License

MIT — see [LICENSE](LICENSE).
