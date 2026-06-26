# fnsd

[中文文档](README.zh-CN.md)

fnsd is a headless Fast Note Sync client. It syncs an Obsidian vault with a Fast Note Sync server without running Obsidian.

It supports one-shot sync and a long-running daemon mode that watches local file changes.

## Features

- Sync notes, files, folders, and supported Obsidian configuration files.
- Watch local vault changes and send them to the server in daemon mode.
- Apply remote changes to the local vault.
- Supports note/file/folder rename during watch mode.
- Supports file chunk upload/download and resumable transfer state.
- Uses SQLite for local sync state.
- Uses protobuf protocol by default.

## Requirements

- A running Fast Note Sync server.
- A websocket sync token from that server.
- Access to the vault directory on disk.

## Install

Build from source:

```powershell
cargo build --release
```

The binary is:

```text
target/release/fnsd
```

## Configuration

Create an example config:

```powershell
fnsd --config fnsd.toml init-config
```

Minimum required settings:

```toml
[server]
url = "https://sync.example.com"
api_token = "your-websocket-sync-token"

[vault]
name = "My Vault"
root = "D:/Obsidian/My Vault"
```

The full example is in [fnsd.example.toml](fnsd.example.toml).

By default, fnsd stores local state in:

```text
.fnsd/state.sqlite
```

If `store.path` is relative, it is resolved from the current working directory. For predictable behavior, set it explicitly in the config.

## Commands

Validate the config:

```powershell
fnsd --config fnsd.toml config check
```

Show local sync state:

```powershell
fnsd --config fnsd.toml status
```

Run one sync pass and exit:

```powershell
fnsd --config fnsd.toml sync once
```

Run as a long-lived watcher:

```powershell
fnsd --config fnsd.toml daemon run
```

## Logging

Console logging defaults to `info`.

```powershell
fnsd --config fnsd.toml --log-level debug daemon run
```

Write logs to a file as well:

```powershell
fnsd --config fnsd.toml --log-file .fnsd/fnsd.log daemon run
```

The same settings can be supplied with environment variables:

```powershell
$env:FNSD_LOG = "debug"
$env:FNSD_LOG_FILE = ".fnsd/fnsd.log"
```

## systemd

For Linux servers, copy the example unit file to `/etc/systemd/system/fnsd.service`:

```powershell
Copy-Item deploy/systemd/fnsd.service /etc/systemd/system/fnsd.service
```

Prepare the config file at `/etc/fnsd/fnsd.toml` and make sure the paths inside it are absolute:

```toml
[vault]
root = "/srv/obsidian/vault"

[store]
path = "/var/lib/fnsd/state.sqlite"
```

Then enable and start the service:

```powershell
systemctl daemon-reload
systemctl enable --now fnsd
```

View logs with:

```powershell
journalctl -u fnsd -f
```

## Environment Variables

Configuration values can be supplied with the `FNSD_` prefix. Nested fields use `__`.

Examples:

```powershell
$env:FNSD_SERVER__URL = "https://sync.example.com"
$env:FNSD_SERVER__API_TOKEN = "your-websocket-sync-token"
$env:FNSD_VAULT__NAME = "My Vault"
$env:FNSD_VAULT__ROOT = "D:/Obsidian/My Vault"
$env:FNSD_STORE__PATH = "D:/Obsidian/My Vault/.fnsd/state.sqlite"
```

List values use commas:

```powershell
$env:FNSD_SCAN__IGNORE_EXTENSIONS = "tmp,bak"
```

## Client Name and Token Restrictions

fnsd sends `client.name = "fnsd"` during the websocket handshake by default. If your Fast Note Sync server token is restricted by client name/type, allow `fnsd` for that token or set a matching value:

```toml
[client]
name = "fnsd"
```

## Docker

Run fnsd with a Docker image:

```powershell
docker run --rm ghcr.io/enximi/fnsd:latest --help
```

Version tags are also available, for example `ghcr.io/enximi/fnsd:v0.1.3`.

### Docker Compose

Copy the compose example:

```powershell
Copy-Item docker-compose.example.yml docker-compose.yml
```

Edit these values:

```yaml
image: ghcr.io/enximi/fnsd:latest
volumes:
  - ./fnsd.toml:/data/fnsd.toml:ro
  - /path/to/your/obsidian-vault:/data/vault
```

In `fnsd.toml`, use container paths:

```toml
[vault]
root = "/data/vault"

[store]
path = "/data/vault/.fnsd/state.sqlite"
```

Start the daemon:

```powershell
docker compose up -d
```

View logs:

```powershell
docker compose logs -f
```

Stop it:

```powershell
docker compose down
```

### Docker Run

Run one sync pass:

```powershell
docker run --rm `
  -v D:/Obsidian/MyVault:/data/vault `
  -v ${PWD}/fnsd.toml:/data/fnsd.toml:ro `
  ghcr.io/enximi/fnsd:latest --config /data/fnsd.toml sync once
```

Run daemon mode:

```powershell
docker run -d --name fnsd --restart unless-stopped `
  -v D:/Obsidian/MyVault:/data/vault `
  -v ${PWD}/fnsd.toml:/data/fnsd.toml:ro `
  -e FNSD_LOG=info `
  -e FNSD_LOG_FILE=/data/vault/.fnsd/fnsd.log `
  ghcr.io/enximi/fnsd:latest --config /data/fnsd.toml daemon run
```

### Build Locally

Build the image from this repository:

```powershell
docker build -t fnsd .
```

Then run it with the local image name:

```powershell
docker run --rm `
  -v D:/Obsidian/MyVault:/data/vault `
  -v ${PWD}/fnsd.toml:/data/fnsd.toml:ro `
  fnsd --config /data/fnsd.toml sync once
```

In Docker, set `vault.root = "/data/vault"` and `store.path = "/data/vault/.fnsd/state.sqlite"`.

## Sync Behavior

The default scan behavior matches the Fast Note Sync plugin convention:

- Vault notes are scanned as Markdown notes.
- Root Obsidian config syncs root JSON files.
- Plugin config syncs `json`, `js`, and `css`.
- Theme config syncs `css` and `json`.
- Custom config directories are scanned recursively.

By default, local paths missing during sync are reported as missing instead of delete. Set this only if you want offline local deletions to propagate:

```toml
[sync]
offline_delete_sync_enabled = true
```

## Protobuf

Protobuf mode is enabled by default:

```toml
[client]
protobuf = true
```

Set it to `false` only if you need JSON websocket frames for compatibility or debugging.
