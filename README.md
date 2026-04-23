# grindd

A small, Rust-based container engine workspace with a CLI and core runtime components.

This project is organized as a Cargo workspace:

- `crates/grindd-cli`: CLI entrypoint and command handling.
- `crates/grindd-core`: Engine, runtime, image/build, storage, and networking modules.

## Current Scope

`grindd` currently provides foundational container-engine building blocks, including:

- Config loading and bootstrap state initialization.
- Image tar loading into a local store.
- Basic container run flow with optional memory/CPU settings.
- Container metadata inspection and lifecycle explanation reports.
- Build system support for `FROM`, `RUN`, `COPY`, and `CMD` instructions.
- Networking and storage orchestration notes and Linux-focused plumbing.

It should be treated as experimental and educational at this stage.

## Requirements

- Rust toolchain (stable) with Cargo.
- Linux host capabilities for namespace/cgroup/network/storage operations.
- On non-Linux platforms, namespace runtime features are expected to return unsupported errors.

## Build

From repository root:

```bash
cargo build
```

## Run CLI

Run the CLI directly through Cargo:

```bash
cargo run -p grindd-cli -- --help
```

## Common Commands

```bash
# Print effective config
cargo run -p grindd-cli -- config

# Start daemon loop (foreground)
cargo run -p grindd-cli -- daemon

# Load an image tar into local store
cargo run -p grindd-cli -- image-load --name alpine --tar ./alpine.tar

# Run a container
cargo run -p grindd-cli -- run --id demo --image alpine -- echo hello

# List containers
cargo run -p grindd-cli -- ps

# Show container logs
cargo run -p grindd-cli -- logs --id demo

# Inspect a container
cargo run -p grindd-cli -- inspect --id demo

# Explain lifecycle plan
cargo run -p grindd-cli -- explain --id demo

# Remove container metadata
cargo run -p grindd-cli -- rm --id demo
```

## Optional Runtime Flags

`run` supports optional resource flags:

- `--memory <bytes>`
- `--cpu-quota <quota>`

Example:

```bash
cargo run -p grindd-cli -- run --id demo --image alpine --memory 134217728 --cpu-quota 50000 -- sh -c "echo hi"
```

## Configuration

By default, the CLI uses internal defaults:

- `state_root`: `/var/lib/grindd`
- `log_level`: `info`
- `cgroup_root`: `/sys/fs/cgroup`
- `bridge_name`: `grindd0`

You can pass a custom JSON config file with `--config`:

```bash
cargo run -p grindd-cli -- --config ./grindd.json config
```

Example config:

```json
{
  "state_root": "/var/lib/grindd",
  "log_level": "info",
  "cgroup_root": "/sys/fs/cgroup",
  "bridge_name": "grindd0"
}
```

## Project Docs

Additional implementation notes are in:

- `docs/phase-build.md`
- `docs/phase-network.md`
- `docs/phase-runtime.md`
- `docs/phase-storage.md`
