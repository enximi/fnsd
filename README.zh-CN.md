# fnsd

[English](README.md)

fnsd 是一个无界面的 Fast Note Sync 客户端。它可以在不启动 Obsidian 的情况下，把本地 Obsidian vault 和 Fast Note Sync 服务器同步。

fnsd 支持一次性同步，也支持长期运行的 daemon 模式，用来监听本地文件变化并实时同步。

## 功能

- 同步笔记、文件、文件夹和支持的 Obsidian 配置文件。
- daemon 模式下监听本地 vault 变化并推送到服务器。
- 接收远程变化并写入本地 vault。
- watch 阶段支持笔记、文件、文件夹重命名。
- 支持文件分片上传/下载和传输断点状态。
- 使用 SQLite 保存本地同步状态。
- 默认使用 protobuf 协议。

## 运行要求

- 已运行的 Fast Note Sync 服务器。
- 服务器生成的 websocket 同步 token。
- 本机能访问的 Obsidian vault 目录。

## 安装

从源码构建：

```powershell
cargo build --release
```

生成的二进制文件：

```text
target/release/fnsd
```

## 配置

生成配置示例：

```powershell
fnsd --config fnsd.toml init-config
```

最少需要配置这些字段：

```toml
[server]
url = "https://sync.example.com"
api_token = "your-websocket-sync-token"

[vault]
name = "My Vault"
root = "D:/Obsidian/My Vault"
```

完整配置示例见 [fnsd.example.toml](fnsd.example.toml)。

默认本地状态文件位置：

```text
.fnsd/state.sqlite
```

如果 `store.path` 使用相对路径，它会基于当前工作目录解析。为了行为稳定，建议在配置中显式设置。

## 命令

校验配置：

```powershell
fnsd --config fnsd.toml config check
```

查看本地同步状态：

```powershell
fnsd --config fnsd.toml status
```

执行一次同步后退出：

```powershell
fnsd --config fnsd.toml sync once
```

长期运行并监听本地变化：

```powershell
fnsd --config fnsd.toml daemon run
```

## 日志

终端日志默认级别是 `info`。

```powershell
fnsd --config fnsd.toml --log-level debug daemon run
```

同时写入日志文件：

```powershell
fnsd --config fnsd.toml --log-file .fnsd/fnsd.log daemon run
```

也可以使用环境变量：

```powershell
$env:FNSD_LOG = "debug"
$env:FNSD_LOG_FILE = ".fnsd/fnsd.log"
```

## systemd

在 Linux 服务器上，可以先把示例 unit 文件复制到 `/etc/systemd/system/fnsd.service`：

```powershell
Copy-Item deploy/systemd/fnsd.service /etc/systemd/system/fnsd.service
```

然后把配置文件放到 `/etc/fnsd/fnsd.toml`，并把其中的路径写成绝对路径：

```toml
[vault]
root = "/srv/obsidian/vault"

[store]
path = "/var/lib/fnsd/state.sqlite"
```

接着启用并启动服务：

```powershell
systemctl daemon-reload
systemctl enable --now fnsd
```

查看日志：

```powershell
journalctl -u fnsd -f
```

## 环境变量

配置项可以通过 `FNSD_` 前缀设置。嵌套字段使用 `__` 分隔。

示例：

```powershell
$env:FNSD_SERVER__URL = "https://sync.example.com"
$env:FNSD_SERVER__API_TOKEN = "your-websocket-sync-token"
$env:FNSD_VAULT__NAME = "My Vault"
$env:FNSD_VAULT__ROOT = "D:/Obsidian/My Vault"
$env:FNSD_STORE__PATH = "D:/Obsidian/My Vault/.fnsd/state.sqlite"
```

列表值使用英文逗号分隔：

```powershell
$env:FNSD_SCAN__IGNORE_EXTENSIONS = "tmp,bak"
```

## 客户端名称和 token 限制

fnsd 默认在 websocket 握手时发送 `client.name = "fnsd"`。如果 Fast Note Sync 服务器上的 token 配置了客户端名称或类型限制，需要允许 `fnsd`，或者在配置里改成和 token 限制匹配的值：

```toml
[client]
name = "fnsd"
```

## Docker

使用 Docker 镜像运行 fnsd：

```powershell
docker run --rm ghcr.io/enximi/fnsd:latest --help
```

也可以使用版本 tag，例如 `ghcr.io/enximi/fnsd:v0.1.3`。

### Docker Compose

复制 compose 示例：

```powershell
Copy-Item docker-compose.example.yml docker-compose.yml
```

修改这些值：

```yaml
image: ghcr.io/enximi/fnsd:latest
volumes:
  - ./fnsd.toml:/data/fnsd.toml:ro
  - /path/to/your/obsidian-vault:/data/vault
```

`fnsd.toml` 中使用容器内路径：

```toml
[vault]
root = "/data/vault"

[store]
path = "/data/vault/.fnsd/state.sqlite"
```

启动 daemon：

```powershell
docker compose up -d
```

查看日志：

```powershell
docker compose logs -f
```

停止：

```powershell
docker compose down
```

### Docker Run

执行一次同步：

```powershell
docker run --rm `
  -v D:/Obsidian/MyVault:/data/vault `
  -v ${PWD}/fnsd.toml:/data/fnsd.toml:ro `
  ghcr.io/enximi/fnsd:latest --config /data/fnsd.toml sync once
```

运行 daemon：

```powershell
docker run -d --name fnsd --restart unless-stopped `
  -v D:/Obsidian/MyVault:/data/vault `
  -v ${PWD}/fnsd.toml:/data/fnsd.toml:ro `
  -e FNSD_LOG=info `
  -e FNSD_LOG_FILE=/data/vault/.fnsd/fnsd.log `
  ghcr.io/enximi/fnsd:latest --config /data/fnsd.toml daemon run
```

### 本地构建

从当前仓库构建镜像：

```powershell
docker build -t fnsd .
```

然后使用本地镜像运行：

```powershell
docker run --rm `
  -v D:/Obsidian/MyVault:/data/vault `
  -v ${PWD}/fnsd.toml:/data/fnsd.toml:ro `
  fnsd --config /data/fnsd.toml sync once
```

Docker 中建议设置：

```toml
[vault]
root = "/data/vault"

[store]
path = "/data/vault/.fnsd/state.sqlite"
```

## 同步行为

默认扫描行为对齐 Fast Note Sync 插件的约定：

- vault 中的 Markdown 文件作为笔记同步。
- Obsidian 根配置只同步根目录下的 JSON 文件。
- 插件配置同步 `json`、`js`、`css`。
- 主题配置同步 `css`、`json`。
- 自定义配置目录会递归扫描。

默认情况下，本地缺失路径会作为 missing 上报，由服务器决定是否重新推送。只有明确希望本地离线删除传播到服务器时，才开启：

```toml
[sync]
offline_delete_sync_enabled = true
```

## Protobuf

默认启用 protobuf：

```toml
[client]
protobuf = true
```

只有在需要 JSON websocket frame 兼容或调试时，才建议改成 `false`。
