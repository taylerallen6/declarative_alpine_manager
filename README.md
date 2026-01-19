# DAM: Declarative Alpine Manager

## Overview
This project is a lightweight, Rust-based tool for managing an Alpine Linux system in a fully declarative way. It allows you to define your system's desired state (e.g., packages, users, groups, services, networking) in a single configuration file, and the tool reconciles the actual system to match that state idempotently, meaning it only applies necessary changes without redundancy.

Inspired by tools like NixOS or Ansible, but tailored for Alpine's minimalism, this program emphasizes direct file editing in Rust for cleanliness and reproducibility, with minimal shell commands. It's designed for users who want secure, resource-efficient setups that are easy to version-control and rollback.

## Key Features
- **Declarative Configuration**: Define your desired system state in a TOML file (e.g., `config.toml` for packages, users, and other domains). The tool generates or edits system files (mostly in /etc) to match.
- **Reconciliation Process**: For each domain (packages, users, etc.):
  - Fetches the current state by reading files directly in Rust.
  - Computes differences in memory (e.g., missing users or packages).
  - Applies changes by writing files atomically, with backups for rollback, and minimal shell for activation (e.g., `apk upgrade`).
- **Idempotency and Dry-Run**: Safe to run repeatedly; includes a `--dry-run` flag to preview changes without writing files.
- **Extensibility**: Add new domains by implementing custom structs (e.g., NetworkingDeclaration) that handle specific files, no complex parsing needed initially.
- **Minimal Footprint**: Compiles to a static binary compatible with Alpine (musl libc), with low runtime overhead and a preference for pure Rust file operations over shell commands.

## Usage
1. **Install Dependencies**: On Alpine, `apk add rust cargo`.
2. **Build**: `cargo build --release --target x86_64-unknown-linux-musl`.
3. **Configure**:
   - Edit `config.toml` with desired states (e.g., package lists, user attributes like username, groups, shell).
4. **Run**:
   - `./target/release/declarative-alpine apply --config config.toml` (applies changes atomically).
   - Add `--dry-run` to simulate without modifications.
   - Other commands: `diff` to show differences.

Example `config.toml`:
```
packages = ["git", "vim"]

users = [
  { username = "tayler", groups = ["wheel"], shell = "/bin/ash", home = "/home/tayler" }
]
```

## Project Structure
- `src/main.rs`: CLI entry, config loading, and reconciliation loop over declarations.
- `src/config.rs`: Parses the desired state from TOML.
- `src/declarations/`: Trait for declarations and custom impls (e.g., `packages.rs` for /etc/apk/world, `users.rs` for /etc/passwd/shadow/group).
- `Cargo.toml`: Dependencies (e.g., clap, serde, toml, regex for parsing if needed).

## Why This Project?
Built for creating the ultimate lightweight declarative Linux on Alpine, secure, minimal, and fully reproducible through direct file manipulation. Ideal for servers, embedded systems, or personal machines where you want Git-versioned configs with minimal external dependencies.

## Contributing
Fork, add new declaration impls, or improve file handling—PRs welcome! Run as root for system changes; test in a VM.

License: Apache-2.0 license.
