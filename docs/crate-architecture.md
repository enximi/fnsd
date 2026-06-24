# Crate 架构规划

本文档记录当前 headless FNS 客户端的 crate 边界，以及进入长期运行模式前建议做的结构调整。

目标不是把 crate 拆得越多越好，而是让每个 crate 的职责足够单一，方便手工验证、局部修改和后续替换实现。

## 当前结论

当前 workspace 的基础分层是合理的：

- `fns-core`：基础领域类型。
- `fns-hash`：兼容原插件的 hash 算法。
- `fns-protocol`：JSON / protobuf / binary frame 的协议模型与编解码。
- `fns-ws-client`：WebSocket 传输适配。
- `fns-vault-fs`：vault 文件系统读写、扫描、路径解析。
- `fns-local-store`：本地同步元数据、hash index、pending 状态。
- `fns-sync-plan`：纯同步请求和操作规划。
- `fns-file-transfer`：文件分片上传/下载的纯传输辅助。
- `fns-sync-apply`：服务端事件和 ack 应用到本地。
- `fns-sync-engine`：一次性同步编排。
- `fns-sync-session`：长期 WebSocket 会话和本地事件增量发送。
- `fns-config`：配置模型和配置加载。
- `fns-client-headless`：CLI 入口。

需要调整的是长期运行相关职责。目前 `fns-sync-engine` 已经承担了连接、扫描、发送同步请求、事件循环、文件传输队列、下载断点、状态保存等职责。对于 `sync once` 还能接受，但如果继续加入 watch、daemon、重连、退避，会变成过大的编排 crate。

## 目标依赖方向

依赖方向应保持单向：

```text
fns-client-headless
  -> fns-daemon
  -> fns-sync-session
  -> fns-sync-engine
  -> fns-sync-apply / fns-sync-plan / fns-file-transfer
  -> fns-protocol / fns-ws-client / fns-vault-fs / fns-local-store
  -> fns-core / fns-hash
```

其中 `fns-daemon` 只做长期运行调度，不应该直接实现协议细节、文件分片、事件应用或 hash 逻辑。长期 WebSocket 会话放在 `fns-sync-session`。

## 建议新增 crate

### fns-vault-watch

职责：

- 封装 `notify` 或其他文件监听实现。
- 监听 vault 根目录变化。
- 复用 `fns-vault-fs` 的忽略规则和路径规则。
- 把系统文件事件归一化为 vault 级别的变化信号。
- 输出“需要同步”的事件，而不是直接决定同步内容。

不做：

- 不连接服务器。
- 不读写 local store。
- 不判断冲突。
- 不做 rename 语义检测。
- 不直接调用 `sync_once()`。

推荐输出模型：

```text
VaultWatchEvent::Changed { path }
VaultWatchEvent::RescanNeeded
```

第一版可以更简单，只输出“发生变化，需要同步”。不要急着在 watcher 层识别 note/file/folder/setting 类型。

### fns-daemon

职责：

- 长期运行主循环和重连退避。
- 启动 `fns-sync-session`。
- 接收 `fns-vault-watch` 的变化信号。
- 把变化信号转发给长期会话。
- 在 session 断开或失败后做指数退避重连。

不做：

- 不自己构造协议请求。
- 不自己处理服务端事件。
- 不自己实现文件分片。
- 不直接修改 vault 文件。
- 不直接修改 local store，除非以后有 daemon 自己的运行状态文件。

### fns-sync-session

职责：

- 保持一个长期 WebSocket 连接。
- 完成 Authorization、ClientInfo、JSON/protobuf 模式切换。
- 启动后执行一次启动同步。
- 持续处理服务端事件和 ack。
- 接收 watcher 事件并做 debounce。
- 将本地变更转换为增量 action，例如 `NoteModify`、`FileUploadCheck`、`FolderModify`、`SettingModify`、删除 action。
- 处理服务端要求的文件上传/下载。

不做：

- 不直接监听文件系统。
- 不做进程级守护和重连退避。
- 不解析配置文件。
- 不重新实现一次性同步扫描规划；启动同步通过当前长期 WebSocket 复用 `fns-sync-engine` 的 authenticated sync 流程。

推荐入口：

```text
Daemon::new(config).run().await
```

CLI 后续只调用 daemon，不把长期运行状态机写进 `main.rs`。

## 建议调整现有 crate

### fns-sync-engine

保留职责：

- `sync_once()` 的完整闭环。
- 建立一次 WebSocket 会话。
- 授权、ClientInfo、JSON/protobuf 模式切换。
- 扫描本地 vault。
- 构造四大类 sync request。
- drain 服务端事件直到四类 sync end 和相关传输完成。
- 调用 `fns-sync-apply` 应用事件。
- 调用 `fns-file-transfer` 执行文件分片。
- 保存 local store。

需要控制的边界：

- 不加入文件监听。
- 不加入 daemon run loop。
- 不加入重连守护进程。
- 不做跨多次运行的全局调度策略。

可调整项：

- `transfer.rs` 现在是一次同步会话内的传输队列，可以暂时留在 `fns-sync-engine`。
- `checkpoint.rs` 是下载断点文件存储。它只服务传输恢复，后续如果继续膨胀，可以迁到 `fns-file-transfer` 或新建 `fns-transfer-state`。
- `event_loop.rs` 目前属于 sync_once 的内部事件循环，暂时留在 `fns-sync-engine` 是合理的。
- 长期运行期间不应反复调用 `sync_once()` 处理每个本地事件；这部分应由 `fns-sync-session` 通过长连接发送增量 action。

### fns-file-transfer

当前职责合适，但可以逐步增强：

- 保留 `UploadPlan`、`DownloadSession`、chunk 校验、chunk 组装。
- 可以接收更通用的 checkpoint trait，但第一版不急。
- 不应该依赖 `fns-ws-client`。
- 不应该决定何时上传或下载。

如果后面 checkpoint 逻辑继续变复杂，可以拆出：

```text
fns-transfer-state
```

但现在不建议马上拆，避免过早抽象。

### fns-sync-apply

当前拆分方向合理：

- 处理 note/file/folder/setting 服务端事件。
- 处理 ack。
- 更新 vault 和 local store。
- 返回 `EventOutcome` 给 engine。

需要保持：

- 不发 WebSocket 请求。
- 不扫描 vault。
- 不做 daemon 调度。
- 不做文件 chunk 传输。

### fns-config

后续需要新增 daemon/watch 配置：

```toml
[daemon]
watch-enabled = true
debounce-ms = 1000
retry-min-seconds = 5
retry-max-seconds = 300
```

字段建议：

- `watch_enabled: bool`
- `debounce_ms: u64`
- `retry_min_seconds: u64`
- `retry_max_seconds: u64`

配置仍然放在 `fns-config`，不要让 daemon crate 自己解析配置文件。

### fns-client-headless

CLI 应继续保持很薄：

- `config check`
- `sync once`
- `daemon run`

日志初始化可以继续放在 CLI crate。业务执行交给 `fns-sync-engine` 或 `fns-daemon`。

## 不建议调整的 crate

### fns-core

继续保持最小领域类型，不要加入文件系统、协议、配置、网络依赖。

### fns-hash

继续保持原插件 hash 兼容实现，不要把 hash index 或同步状态放进来。

### fns-protocol

继续只做协议类型和编解码。即使 protobuf 映射比较长，也不要让它依赖 engine/store/vault。

### fns-ws-client

继续只做 WebSocket 收发和 frame 转换。不要把重连策略放进来；重连属于 daemon 或更上层运行时策略。

### fns-vault-fs

继续负责 vault 文件系统读写和扫描规则。watcher 可以依赖它，但不要把 watcher 强塞进这个 crate；扫描和监听是两个不同职责。

### fns-local-store

继续只持久化同步状态。daemon 运行状态如果以后需要保存，优先单独设计，不要把所有运行时状态都塞进 local store。

### fns-sync-plan

继续保持纯 planning。不要引入文件系统、WebSocket 或 store。

## 推荐迁移顺序

1. 新增 `fns-vault-watch`。
   - 先输出简单变化信号。
   - 使用现有 scan ignore 规则。
   - 不做 rename 检测。

2. 新增 `fns-daemon`。
   - 启动并重连 `fns-sync-session`。
   - 转发 watcher 事件。
   - 实现失败重试。

3. 扩展 `fns-config`。
   - 增加 `[daemon]` 配置段。
   - 默认启用 watch。

4. 扩展 CLI。
   - 增加 `daemon run`。
   - 继续保留 `sync once` 作为调试和手工同步入口。

5. 观察 `fns-sync-engine`。
   - 如果 `transfer.rs` 或 `checkpoint.rs` 后续继续变大，再考虑拆 `fns-transfer-state`。
   - 在没有明确复杂度之前，不先拆。

## 长期运行模式的第一版行为

建议第一版 daemon 行为如下：

1. 启动后加载配置。
2. 启动文件监听。
3. 启动 `fns-sync-session` 并建立长期 WebSocket。
4. session 启动后执行一次启动同步。
5. 文件变化后转发给 session。
6. session 对本地事件做 debounce，并通过同一个 WebSocket 发送增量 action。
7. 如果 session 失败或断开，daemon 按 retry 配置退避重连。

这样可以先覆盖 headless 客户端最重要的长期运行需求，同时避免在 watcher 阶段过早做复杂 rename 检测。

## Rename 策略

长期运行第一版不做本地 rename 检测。

原因：

- 一次性同步阶段只能看到前后两个快照，hash 相同不一定表示 rename，也可能是复制、删除后重建或多个同内容文件。
- 文件系统 watcher 的 rename 事件在不同平台上语义不完全一致。
- 原项目主要依赖插件运行时事件直接知道 rename；headless 客户端没有编辑器事件源。

第一版策略：

- 本地 rename 按“旧路径 missing/deleted + 新路径 modify”处理。
- 默认不启用离线删除同步时，旧路径进入 missing，由服务端决定是否回推。
- 启用离线删除同步时，旧路径进入 deleted。

如果后续确实需要 rename 检测，可以单独设计一个可关闭的启发式策略，不放进第一版 daemon。
