# Contributing to OpenUSB

Thanks for your interest in contributing to OpenUSB!

## Development Setup

1. Install [Rust](https://rustup.rs/) (1.85+)
2. Clone the repo:
   ```bash
   git clone https://github.com/kman1898/usb-passthrough.git
   cd usb-passthrough
   ```
3. Build all crates:
   ```bash
   cargo build
   ```
4. Run tests:
   ```bash
   cargo test
   ```

## Project Structure

- `server/openusbd/` — Server daemon (runs on Raspberry Pi / Linux)
- `shared/` — Shared types, protocol definitions, USB ID database
- `client/common/` — Shared client library (discovery, connection, auto-connect)
- `cli/` — Command-line tool
- `client/windows/` — Windows tray application (Phase 2)
- `web-dashboard/` — React web UI (Phase 4)

## Pull Requests

1. Fork the repo and create a feature branch
2. Make your changes
3. Run `cargo fmt` and `cargo clippy`
4. Run `cargo test`
5. Open a PR with a clear description of what changed and why

## Code Style

- Run `cargo fmt` before committing
- Address all `cargo clippy` warnings
- Write tests for new functionality
- Keep commits focused — one logical change per commit

## Reporting Issues

Open an issue on GitHub with:
- What you expected to happen
- What actually happened
- Steps to reproduce
- Your OS and Rust version
