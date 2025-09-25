# WordMesh 身份认证模块技术设计

## 1. 范围与约束

- 范围：注册、登录、（可选）令牌刷新、获取当前用户资料。
- 约束：
  - 数据库与根《Technical_Design.md》一致：仅使用 `users` 表（`id`, `username`, `password`, `created_at`）。
  - 遵循仓库约定：分层结构、`Result<T, AppError>`、`tracing`、统一响应 `{ data, meta, error }`、`request_id`、慢请求 >500ms 记 `warn`、OpenAPI。
  - Rust stable；启用 clippy、rustfmt；依赖经评审并锁定版本；`cargo deny`/`audit` 通过。

## 2. 模块分层与目录

- `controller/auth_controller.rs`：挂载 `/api/v1/auth/*`，接入 DTO，调用 service，返回统一响应。
- `dto/auth.rs`：`RegisterRequest`, `LoginRequest`, `RefreshRequest?`, `ProfileResponse`，使用 `validator` 做字段约束。
- `service/auth_service.rs`：注册/登录/刷新/资料获取编排；密码校验；令牌签发与校验；发布事件。
- `repository/user_repository.rs`：围绕 `users` 表的读写；查询用户名是否存在、按用户名取用户、创建用户。
- `middleware/auth_guard.rs`：解析 `Authorization: Bearer <token>`，注入 `UserClaims` 到请求上下文。
- `util/password.rs`：密码哈希与校验封装（策略参数）。
- `util/token.rs`：访问/刷新令牌生成与校验（算法、TTL、Claims）。

备注：业务代码不得置于 `main.rs`。

## 3. 配置设计（Settings）

- `auth.enabled: bool`（默认 true）
- `auth.jwt.algorithm: string`（支持 HS256 或 RS256，默认由配置决定，可切换）
- `auth.jwt.access_ttl_secs: u64`（默认 3600）
- `auth.jwt.refresh_ttl_secs: u64`（默认 604800；设置为 0 或移除即可关闭刷新令牌）
- `auth.jwt.secret: string`（HS 时必需）或 `auth.jwt.private_key/public_key: string`（RS 时必需）
- `auth.password.min_length: u8`（默认 8）
- `auth.password.require_complexity: bool`（默认 false）

校验：算法与密钥组合合法；TTL > 0；密码策略范围有效；敏感项从环境变量读取；当刷新 TTL 关闭时，刷新接口将拒绝请求。

## 4. 数据模型与持久化

仅使用 `users` 表：

- `id BIGSERIAL PRIMARY KEY`
- `username VARCHAR(255) UNIQUE NOT NULL`
- `password VARCHAR(255) NOT NULL`（哈希）
- `created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`

索引：`UNIQUE(username)` 满足按用户名查询的高频场景。

## 5. 接口契约（技术）

- POST `/api/v1/auth/register`
  - Req: `{ username, password }`
  - 校验：用户名唯一、长度 3–32、字符集；密码长度 ≥8 与复杂度策略。
  - Resp.data: `{ id, username, created_at }`
- POST `/api/v1/auth/login`
  - Req: `{ username, password }`
  - Resp.data: `{ access_token, refresh_token? }`
  - 失败：统一“凭证无效”业务错误。
- POST `/api/v1/auth/refresh`（默认启用，无状态验证）
  - Req: `{ refresh_token }`
  - Resp.data: `{ access_token }`
- GET `/api/v1/auth/profile`（受保护）
  - Auth: `Authorization: Bearer <access_token>`
  - Resp.data: `{ id, username, created_at }`

所有响应使用 `{ data, meta, error }`，错误码遵循全局规范，HTTP 始终 200。

## 6. 错误处理与映射

- 使用 `AppError` 聚合；新增 `BusinessError::Auth(AuthFlowError)`（建议）：
  - `InvalidCredentials`, `TokenInvalid`, `TokenExpired`
- 业务码参考：`4011` 无效凭证，`4012` Token 过期，`4013` Token 无效，`4001` 校验失败，`5000` 内部错误。
- 日志：鉴权失败与校验失败用 `warn`，系统错误 `error`；均带 `request_id`，成功鉴权可附 `user_id`。

## 7. 中间件与横切

- `RequestId`：生成并贯穿日志与响应。
- `AuthGuard`：提取 Bearer → 验证 → 注入 `UserClaims`；失败返回统一业务错误。
- 慢请求：>500ms 记录 `warn`，包含路由、user_id（可选）。

## 8. 安全设计

- 密码策略：最小长度 ≥8；复杂度可配置；注册/登录均执行校验。
- 哈希：在 `util/password` 封装 `hash/verify`，禁止记录明文。
- 令牌：
  - `access_token`: 短效（默认 3600s）。
  - `refresh_token`: 默认启用、长效（默认 7d），无状态校验（仅验签与过期），不维护黑名单。
- Claims：`sub`(user_id), `iat`, `exp`, `scope`, `request_id`。
- 脱敏：日志不输出密码与令牌，仅输出长度或指纹。

## 9. 事件与可观测性

- 领域事件：`UserRegistered`, `UserLoggedIn`, `TokenRefreshed`（当前阶段仅记录日志/指标；未来如需消费需追加实现）。
- 指标：登录成功/失败计数、注册计数、刷新计数；认证路由耗时直方图。
- 日志：JSON 输出；异常链路保留 `request_id`。

## 10. 测试与 CI

- 单元：service 核心分支与边界；密码与令牌工具属性测试。
- 集成：handler 端到端（注册 → 登录 → 资料 → 刷新）。
- 合同：响应结构与错误码稳定性。
- CI：`fmt --check`, `clippy -D warnings`, `test --workspace`, `deny`, `audit`。

## 11. 性能目标与调优

- 目标：QPS ≥ 300；P95 < 50ms（不含网络）。
- 调优：根据环境调整哈希成本与签名算法；缓存解析过的 Claims 于请求作用域。

## 12. 风险与演进

- 无状态刷新不可单点撤销；如需登出/撤销，需 ADR 引入会话与黑名单存储。
- 未来第三方登录、密码重置与权限将引入新表与流程，需更新根设计与迁移脚本。
