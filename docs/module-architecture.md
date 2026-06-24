# 模块架构

本文档记录当前“单二进制 crate + 内部模块”的层级关系。

项目最终交付物是一个 headless 客户端应用，因此仓库使用一个普通二进制 crate。原来的 crate 边界已经转换为模块边界；模块之间仍然要保持单向依赖，避免把同步流程、文件系统、协议和运行调度混在一起。

## 目录结构

当前结构如下：

```text
src/
  main.rs
  config/
  core/
  hash/
  protocol/
  ws/
  vault/
    fs/
    watch/
  store/
  sync/
    plan/
    apply/
    transfer/
    engine/
    session/
  daemon/
```

## 依赖方向

依赖应保持从高层到低层：

```text
main
  -> daemon
  -> sync::session
  -> sync::engine
  -> sync::{apply, plan, transfer}
  -> ws / vault / store / protocol / config
  -> core / hash
```

更具体地说：

- `main` 只负责 CLI、日志初始化、加载配置并调用业务入口。
- `daemon` 负责长期运行、watch 转发、session 重连退避。
- `sync::session` 负责长期 WebSocket、启动同步、watch 增量发送、服务端事件循环、回声抑制。
- `sync::engine` 负责 `sync once` 的完整闭环。
- `sync::apply` 负责把服务端事件和 ack 应用到 vault 与 store。
- `sync::plan` 负责构造同步请求、中间资源模型和待发送操作。
- `sync::transfer` 负责文件分片上传、下载、校验和断点辅助。
- `ws` 负责 WebSocket 收发和 frame 转换。
- `vault::fs` 负责 vault 路径、扫描、读写和忽略规则。
- `vault::watch` 负责文件系统监听事件归一化。
- `store` 负责本地 hash index、sync time、pending、checkpoint 持久化。
- `protocol` 负责 JSON、protobuf、binary frame 的协议模型和编解码。
- `config` 负责配置模型和加载。
- `hash` 负责兼容原项目的 hash 算法。
- `core` 只放基础领域类型和通用错误。

## 禁止依赖方向

低层模块不应该反向依赖高层模块：

- `core` 不依赖任何业务模块。
- `hash` 不依赖 `sync`、`vault`、`store`、`protocol`。
- `protocol` 不依赖 `sync`、`vault`、`store`、`daemon`。
- `store` 不依赖 `sync::engine`、`sync::session`、`daemon`。
- `vault::fs` 不依赖 `sync::engine`、`sync::session`、`daemon`。
- `vault::watch` 不依赖 `sync` 或 `daemon`。
- `sync::apply` 不发送 WebSocket 请求，不启动扫描，不做 daemon 调度。
- `sync::plan` 不读写文件系统、不访问 WebSocket、不修改 store。
- `ws` 不包含重连策略；重连属于 `daemon`。

单 crate 内部，Rust 不再通过 crate 依赖阻止反向依赖，所以需要用目录结构、`pub(crate)` 可见性和代码 review 维护这些边界。

## 已整理的边界

### vault::fs 与 sync::plan

`vault::fs` 不应该依赖 `sync::plan`。文件系统层只表达本地文件系统事实，不表达同步决策。

当前资源扫描模型已经下沉到 `core`：

- `TextResource`
- `NoteResource`
- `SettingResource`
- `FileResource`
- `FolderResource`
- `DeletedResource`
- `SyncBatch`

`vault::fs` 返回这些基础资源类型，`sync::plan` 继续使用并 re-export 它们，用来构造同步请求。这样可以避免 `vault::fs -> sync::plan` 的反向依赖。

## 需要特别注意的边界

### sync::session 与 sync::engine

`sync::session` 可以复用 `sync::engine` 的 authenticated sync 流程来做启动同步。

但依赖只能是：

```text
sync::session -> sync::engine
```

不能让 `sync::engine` 反过来依赖 `sync::session`，否则 `sync once` 会被长期运行逻辑污染。

### pending 与回声抑制

这两个概念要继续分开：

- `store::pending` 记录本地发出的 modify/delete/rename/upload，等待服务端 ack 后提交或清理。
- `sync::session` 的回声抑制处理远程写入落盘后触发的 watcher 事件。

不要把远程操作写入 pending，也不要把 pending 当成唯一的回声抑制机制。

## 维护规则

- 新模块优先放到现有层级中，不为很小的职责重新创建 crate。
- 底层模块新增 API 时，应先确认调用方是否真的需要，不要为了上层流程把业务编排塞进底层。
- 调整同步行为时，应优先在 `sync::engine`、`sync::session`、`sync::apply` 中完成，不要让 `protocol`、`store`、`vault::fs` 反向了解上层流程。
- 结构调整应先通过 `cargo check` 和现有端到端脚本，再考虑提交。
