目标：统一 WordMesh 后端开发流程，保证可维护性、可测试性和可观测性
技术栈约定：Rust stable toolchain + Cargo；必须启用 Clippy 与 rustfmt；配置 .cargo/config.toml 固定镜像和构建参数
项目结构：维持 src 下按 config、middleware、domain、dto、event、util、handlers 等模块分层；公共常量与类型放在 src/lib.rs 或 src/shared；禁止将业务代码放入 main.rs
配置管理：使用 config crate 合并多环境配置；默认读取 config/{env}.toml；敏感信息读取环境变量；提供 Settings::from_env() 和 Settings::validate()
依赖治理：新增依赖需经评审；锁定版本到 Cargo.lock；启用 cargo deny 检查许可与安全；避免直接使用不稳定 API
编码规范：遵循 Rust 2018 edition；模板：错误处理用 Result<T, AppError>；日志使用 tracing; 序列化使用 serde；限制 unsafe，需评审和注释
错误处理：统一 AppError 枚举携带 kind、message、context；对外返回统一响应结构 { request_id, status, error }；日志区分 error/ warn/ info/ debug
请求中间件：强制注入 request_id；记录开始/结束时间；对慢请求（>500ms）输出 warn；所有 handler 必须验证输入 DTO
数据库与缓存：使用连接池管理，声明池大小和超时；事务通过 async fn 包裹；缓存需定义失效策略和回源逻辑
API 规范：RESTful 命名；路径使用复数资源；响应格式 { data, meta, error }；启用 OpenAPI 文档自动生成；标注幂等性和权限要求
测试策略：cargo test 覆盖单元/集成测试；关键路径需编写 property-based test；提供 API contract 测试（例如 httpmock）；CI 上运行 cargo fmt --check, cargo clippy -- -D warnings, cargo test --workspace
可 observability：内置 tracing_subscriber 输出 JSON 日志；接入 metrics 端点（Prometheus）；关键事件通过事件总线记录；异常链路保留 request_id
安全要求：输入校验、防止 SQL 注入（使用参数化查询）；开启 TLS；敏感日志脱敏；引入依赖前运行 cargo audit
发布流程：上线前通过 staging 验证、性能压测与回滚预案；版本号遵循 SemVer；生成变更日志；通过 CI/CD 自动部署
文档：维护 docs/architecture.md、docs/api.md、docs/runbook.md；代码注释使用 ///；复杂逻辑配合 ADR（Architecture Decision Record）
开发守则：开启 VSCode/IDE rustfmt on save；PR 需至少一名 reviewer；描述变更、测试结果与风险；合并前确保无 TODO 余留
