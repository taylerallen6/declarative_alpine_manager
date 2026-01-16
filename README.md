# DAM: Declarative Alpine Manager

## Overview
This project is a lightweight, Rust-based tool for managing an Alpine Linux system in a fully declarative way. It allows you to define your system's desired state (e.g., packages, users, services, networking) in configuration files, and the tool reconciles the actual system to match that state idempotently—meaning it only applies necessary changes without redundancy.

Inspired by tools like NixOS or Ansible, but tailored for Alpine's minimalism, this program keeps things simple, secure, and resource-efficient. It's designed for users who want reproducible setups without bloat, and it's extensible for custom states or even switching distros/package managers.

## Key Features
- **Declarative Configuration**: Define what your system should look like in TOML files (e.g., `config.toml` for desired states, per-state files in `./states/` for behavior like commands and parsing).
- **Reconciliation Process**: For each state (packages, users, services, etc.):
  - Fetches the current system state (e.g., via shell commands like `apk info`).
  - Computes differences (missing items, extras, or mismatches).
  - Applies minimal changes (e.g., `apk add` for missing packages).
- **Idempotency and Dry-Run**: Safe to run repeatedly; includes a `--dry-run` flag to preview changes.
- **Extensibility**: Add new states by dropping a TOML file—no code changes needed. Easily swap commands (e.g., for different package managers).
- **Minimal Footprint**: Compiles to a static binary compatible with Alpine (musl libc), with low runtime overhead.

## Usage
1. **Install Dependencies**: On Alpine, `apk add rust cargo`.
2. **Build**: `cargo build --release --target x86_64-unknown-linux-musl`.
3. **Configure**:
   - Edit `config.toml` with desired states (e.g., packages list, hostname).
   - Customize `./states/*.toml` for reconciliation logic (commands, diff types).
4. **Run**:
   - `./target/release/declarative-alpine apply --config config.toml` (applies changes).
   - Add `--dry-run` to simulate.
   - Other commands: `diff` to show differences.

Example `config.toml`:
```
[packages]
items = ["git", "vim"]

[hostname]
value = "minimal-alpine"
```

## Project Structure
- `src/main.rs`: CLI entry and reconciliation loop.
- `src/config.rs`: Parses desired states.
- `src/reconcilers/`: Generic reconciler logic interpreting per-state configs.
- `Cargo.toml`: Dependencies (e.g., clap, serde, toml, regex).

## Why This Project?
Built for creating the ultimate lightweight declarative Linux on Alpine—secure, minimal, and fully reproducible. Ideal for servers, embedded systems, or personal machines where you want Git-versioned configs without heavy tools.

## Contributing
Fork, add states, or improve parsing—PRs welcome! Run as root for system changes; test in a VM.

License: Apache-2.0 license.
