# Crate 规划

本文档记录 FNS 无头客户端的 crate 边界。每个 crate 都应该保持功能单一、
接口容易测试，并避免依赖无关的运行时细节。

## 已存在的 Crate

### `fns-client-headless`

状态：已存在

定位：命令行和 daemon 入口。

职责：

- 解析 CLI 参数。
- 加载配置。
- 初始化日志和运行时服务。
- 把同步引擎和具体实现组装起来。
- 返回清晰的退出状态码和面向用户的错误信息。

不应该做：

- 实现协议编码或解码。
- 实现同步决策逻辑。
- 直接读写 vault 文件，除非是通过底层 crate。
- 拥有 WebSocket 或 HTTP 协议行为。

测试方式：

- 对 CLI 参数解析做 smoke test。
- 命令测试中使用假的同步服务。
- 尽量把行为放到库 crate 中，让这个 crate 保持很薄。

## 计划新增的 Crate

### `fns-core`

定位：低依赖的共享领域类型。

职责：

- 定义核心值类型，例如 vault 名称、vault 内路径、路径哈希、内容哈希、
  本地时间戳和远端时间戳。
- 定义不依赖传输、存储或 UI 的共享错误类型。
- 为核心值提供轻量校验。

不应该做：

- 访问文件系统。
- 访问网络。
- 了解 CLI 参数或配置文件。
- 实现 FNS 同步动作。

测试方式：

- 单元测试构造函数、校验、排序和类型转换。
- 优先使用小型表驱动测试。

### `fns-protocol`

定位：FNS 线协议类型和帧编码。

职责：

- 定义 WebSocket action 名称。
- 定义 notes、files、folders、settings、auth 和 client info 的请求/响应 DTO。
- 编码和解码 `Action|JSON` 格式的文本帧。
- 后续加入 protobuf 支持时，放置生成类型或映射逻辑。
- 在合适的地方把协议 DTO 转换为 `fns-core` 的领域类型。

不应该做：

- 打开 WebSocket 连接。
- 重试请求。
- 读取本地 vault 文件。
- 决定本地数据和远端数据谁胜出。

测试方式：

- 对帧编码和解码做 golden test。
- 用 FNS 文档中的已知 JSON 示例做兼容性测试。
- 测试非法 action 和非法 payload 的错误处理。

### `fns-hash`

状态：已创建

定位：确定性的哈希工具。

职责：

- 计算路径哈希。
- 计算笔记和文件的内容哈希。
- 在协议需要时，对路径做规范化后再哈希。
- 对齐原插件的 JavaScript 32 位滚动哈希：文本按 UTF-16 code unit
  计算，二进制按字节计算，大文件按前 5MB 和后 5MB 采样计算。

不应该做：

- 扫描目录。
- 读取文件，除非确实需要一个很窄的流式辅助函数。
- 了解同步状态或远端时间戳。

测试方式：

- golden hash 测试。
- Windows 和 Unix 风格路径分隔符的规范化测试。
- 测试重复调用时输出稳定。

### `fns-local-store`

定位：持久化的本地同步元数据。

职责：

- 按 vault 和资源类型保存最后同步时间戳。
- 在需要时保存本地资源索引。
- 跟踪本地已删除但尚未被服务端确认的资源。
- 保存崩溃恢复所需的 pending operations。

不应该做：

- 决定同步冲突结果。
- 连接服务端。
- 读写笔记正文。

测试方式：

- 使用临时数据库或内存 store。
- 引入 schema 后测试迁移。
- 测试带 pending operations 的崩溃恢复场景。

### `fns-vault-fs`

状态：已创建

定位：本地 vault 文件系统适配器。

职责：

- 从 vault 目录扫描笔记、附件、文件夹和设置。
- 读取和写入笔记内容。
- 读取和写入附件字节。
- 删除和重命名本地资源。
- 在系统支持时设置修改时间。
- 执行 vault 路径安全检查，确保操作不能逃出 vault 根目录。
- 按 `.md`、配置目录和普通文件把资源分类为 note、setting、file 和 folder。

不应该做：

- 编码 FNS WebSocket 帧。
- 保存同步时间戳。
- 决定冲突策略。
- 解析 CLI 参数。

测试方式：

- 使用临时目录。
- 测试路径穿越拒绝。
- 测试嵌套目录和忽略路径下的扫描结果。
- 测试写入、删除、重命名和 mtime 行为。

### `fns-ws-client`

定位：WebSocket 传输客户端。

职责：

- 连接 FNS WebSocket endpoint。
- 发送鉴权帧和 client info 帧。
- 发送和接收协议帧。
- 处理文件传输所需的二进制分片帧。
- 在需要时提供重连和超时基础能力。

不应该做：

- 检查本地 vault 文件。
- 决定同步计划。
- 持久化同步元数据。
- 直接解析 CLI 配置。

测试方式：

- 使用本地 mock WebSocket server。
- 测试鉴权成功和失败路径。
- 测试帧往返。
- 测试二进制分片帧。
- 测试重连和超时行为。

### `fns-http-client`

定位：非 WebSocket 操作的 REST API 客户端。

职责：

- 在需要时处理登录或 token 相关 API 调用。
- 拉取用户、vault、版本或辅助信息。
- 把 REST 错误包装为有类型的客户端错误。

不应该做：

- 实现 WebSocket 同步。
- 扫描本地文件。
- 决定同步冲突结果。

测试方式：

- 使用 mock HTTP server。
- 测试请求路径、header、token 处理和错误映射。

### `fns-sync-plan`

状态：已创建

定位：纯同步决策逻辑。

职责：

- 比较本地快照和远端同步消息。
- 生成明确的同步操作，例如上传、下载、删除、重命名、更新时间或标记冲突。
- 保持冲突策略确定且容易测试。
- 避免任何副作用。
- 把本地快照组装成 note、file、folder 和 setting 四类 sync request。
- 把服务端同步消息规整成本地执行层可以消费的操作枚举。

不应该做：

- 读写文件。
- 连接网络。
- 持久化状态。
- 启动异步任务。

测试方式：

- 使用小 fixture 做纯单元测试。
- 分别覆盖 note、file、folder 和 setting 的计划生成。
- 修改冲突策略前先补回归测试。

### `fns-sync-engine`

定位：同步流程编排。

职责：

- 协调 vault 扫描、WebSocket 请求、同步计划、本地文件操作和本地元数据更新。
- 执行一次性同步。
- 后续执行 watch 或 daemon 同步循环。
- 依赖通过 trait 隔离，方便使用 fake 实现测试 engine。

不应该做：

- 包含协议 DTO 定义。
- 包含低层 WebSocket 实现。
- 直接解析 CLI 参数。
- 隐藏本应属于 `fns-sync-plan` 的同步决策。

测试方式：

- 使用 fake transport、fake vault 和 fake store。
- 不依赖真实网络测试完整流程。
- 保留少量集成测试覆盖端到端行为。

### `fns-config`

定位：配置加载和校验。

职责：

- 加载配置文件。
- 合并环境变量和 CLI 覆盖项。
- 解析路径。
- 校验必要设置，例如 server URL、token、vault name 和 vault root。

不应该做：

- 打开 WebSocket 连接。
- 执行同步操作。
- 读写 vault 内容。

测试方式：

- 测试 TOML 解析。
- 测试覆盖优先级。
- 测试校验错误。
- 测试默认路径解析。

## 依赖方向

推荐依赖流向：

```text
fns-client-headless
  -> fns-config
  -> fns-sync-engine
       -> fns-sync-plan
       -> fns-ws-client
       -> fns-vault-fs
       -> fns-local-store
       -> fns-protocol
       -> fns-core
```

低层 crate 不应该依赖高层 crate。具体来说：

- `fns-core` 应该处在最底层。
- `fns-protocol` 可以依赖 `fns-core`。
- `fns-sync-plan` 可以依赖 `fns-core` 和 `fns-protocol`。
- `fns-ws-client`、`fns-vault-fs`、`fns-local-store` 等运行时 crate
  应该能在测试中被 fake 实现替换。
- `fns-client-headless` 应该保持为最薄的一层。

## 初始实现顺序

1. `fns-core`
2. `fns-protocol`
3. `fns-hash`
4. `fns-sync-plan`
5. `fns-vault-fs`
6. `fns-ws-client`
7. `fns-local-store`
8. `fns-sync-engine`
9. `fns-config`
10. `fns-client-headless`
