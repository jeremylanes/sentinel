# Sentinel

Sentinel is a lightweight, high-performance file system watcher written in Rust. It monitors specified files and directories for changes and automatically restarts the associated `systemd` services. This tool is ideal for development environments or production servers where configuration changes need to be applied immediately without manual intervention.

## Features

- **Efficient Monitoring**: built on top of the robust `notify` crate to handle file system events efficiently.
- **Recursive Watching**: monitors directories recursively, ensuring changes in subdirectories are detected.
- **Many-to-Many Mapping**: supports configuring multiple paths triggering multiple services.
- **Smart Rate Limiting**: implements a per-service debounce mechanism (2 seconds) to prevent rapid, spammy restarts during batch file updates.
- **Low Resource Usage**: designed with Rust to be minimal and fast.

## Installation

### Prerequisites

- **Rust**: Ensure you have a working Rust toolchain installed. You can install it via [rustup](https://rustup.rs/).
- **Systemd**: This tool is designed to interact with `systemd` and requires `systemctl` to be available on the path.

### Building from Source

1. Clone the repository:
   ```bash
   git clone git@github.com:jeremylanes/sentinel.git
   cd sentinel
   ```

2. Build the release binary:
   ```bash
   cargo build --release
   ```

3. Install the binary (example):
   ```bash
   sudo cp target/release/sentinel /usr/local/bin/
   ```

## Configuration

Sentinel reads its configuration from `/etc/sentinel.toml`. You must create this file before running the service.

### Configuration Format

The configuration file uses TOML format. You can define multiple watcher items. Each item consists of a list of `paths` to watch and a list of `services` to restart when a change is detected.

### Example `/etc/sentinel.toml`

```toml
[watchers]

[[watchers.item]]
paths = ["/etc/nginx/nginx.conf", "/etc/nginx/conf.d"]
services = ["nginx"]

[[watchers.item]]
paths = ["/opt/myapp/config.yaml", "/opt/myapp/plugins/"]
services = ["myapp", "myapp-worker"]
```

In this example:
- Modification of Nginx configuration files will restart the `nginx` service.
- Changes to `myapp` configuration or plugins will restart both `myapp` and `myapp-worker` services.

## Usage

Run Sentinel as root (required to restart system services):

```bash
sudo sentinel
```

Upon startup, Sentinel will:
1. Parse `/etc/sentinel.toml`.
2. Set up watchers for all valid paths.
3. Enter a monitoring loop, listening for file system events.

### Running as a Systemd Service

To ensure Sentinel runs continuously, you can set it up as a systemd service itself.

1. Create a service file `/etc/systemd/system/sentinel.service`:

   ```ini
   [Unit]
   Description=Sentinel File Watcher
   After=network.target

   [Service]
   ExecStart=/usr/local/bin/sentinel
   Restart=always
   User=root

   [Install]
   WantedBy=multi-user.target
   ```

2. Enable and start the service:

   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable --now sentinel
   ```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
