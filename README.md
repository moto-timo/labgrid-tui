# labgrid-tui

A [k9s](https://k9scli.io/)-style terminal UI for monitoring
[labgrid](https://labgrid.readthedocs.io/) infrastructure. Shows places,
resources, and exporters from a labgrid coordinator in real time.

Built with [ratatui](https://ratatui.rs/) and
[tonic](https://github.com/hyperium/tonic) (gRPC).

## Requirements

- Rust 1.70+ (2021 edition)
- Protocol Buffers compiler (`protoc`)
- A running labgrid coordinator (tested against labgrid 25.0.1)

### Installing protoc

On Debian/Ubuntu:

    sudo apt install protobuf-compiler

On Fedora:

    sudo dnf install protobuf-compiler

On macOS (Homebrew):

    brew install protobuf

## Building

    cargo build --release

The binary is at `target/release/labgrid-tui`.

## Usage

Point the TUI at your labgrid coordinator using the `LG_COORDINATOR`
environment variable or the `-c` flag:

    export LG_COORDINATOR=10.0.0.147:20408
    labgrid-tui

    # or explicitly
    labgrid-tui -c http://10.0.0.147:20408

    # with logging to file
    labgrid-tui --log-file /tmp/lgtui.log

Bare `host:port` values are automatically normalized to `http://host:port`.
WebSocket URLs (`ws://host:port/ws`) are also supported for coordinators
that serve gRPC over WebSocket.

### Keybindings

| Key       | Action                          |
|-----------|---------------------------------|
| `1`       | Places view                     |
| `2`       | Resources view                  |
| `3`       | Exporters view                  |
| `j` / `k` | Move down / up                 |
| `/`       | Filter                          |
| `Esc`     | Clear filter / close panel      |
| `Enter`   | Open detail panel               |
| `?`       | Help                            |
| `q`       | Quit                            |

## Configuration

An optional TOML config file can be placed at
`~/.config/labgrid-tui/config.toml`:

```toml
[coordinator]
url = "http://10.0.0.147:20408"

[ui]
tick_rate_ms = 250
```

The CLI flag takes priority over the config file, which takes priority over
the `LG_COORDINATOR` environment variable.

## Platform notes

### Linux

Tested on Ubuntu 24.04. Build and network connectivity work as expected.

### macOS

**Network connectivity issues:** Rust-compiled binaries may fail to make
outbound TCP connections to remote hosts, even when tools like `curl`, `nc`,
and `ping` work fine. Symptoms include "No route to host" errors from the
TUI while the coordinator is otherwise reachable.

This appears to be related to the macOS application firewall or security
policy blocking unsigned binaries. Ad-hoc code signing
(`codesign -s - target/release/labgrid-tui`) has been tried without success.
Cross-compiling on Linux or running inside a container may be more reliable.

If you find a fix for macOS, please contribute it.

## Proto file

The gRPC proto file at `proto/labgrid-coordinator.proto` is from the
upstream labgrid project (25.0.1). If you need to update it:

    curl -o proto/labgrid-coordinator.proto \
      https://raw.githubusercontent.com/labgrid-project/labgrid/master/labgrid/remote/proto/labgrid-coordinator.proto

## License

MIT
