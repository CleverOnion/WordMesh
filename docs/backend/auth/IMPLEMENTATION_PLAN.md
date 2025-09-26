# WordMesh 身份认证模块开发任务规划

## 1. 准备阶段

- [x] 同步文档
  - [x] 阅读 `docs/backend/auth/REQUIREMENTS.md`
  - [x] 阅读 `docs/backend/auth/TECHNICAL_DESIGN.md`
  - [x] 标记潜在开放问题并与团队确认
    - 刷新令牌：默认启用无状态刷新，无需维护黑名单。
    - JWT：算法可配置（HS256/RS256），后续可按环境选择；密钥轮换待运营流程确认。
    - 事件：当前仅记录日志/指标，未来可能接入下游消费。
- [x] 配置支撑
  - [x] 在 `config/settings.rs` 添加 `AuthSettings`
  - [x] 为 `Settings::validate()` 增加认证配置校验
  - [x] 更新各环境配置文件的占位键值
- [x] 错误体系统一
  - [x] 扩展 `util::error`，增加 `BusinessError::Auth`
  - [x] 定义 `AuthFlowError` 及业务码映射
  - [x] 为错误新增单元测试（验证 ResponseError 输出）

## 2. 数据访问与领域模型

- [x] 领域建模
  - [x] 定义 `domain::user::User` 聚合结构
  - [x] 编写 `HashedPassword` 值对象（含校验规则）
  - [x] 补充必要的构造/验证逻辑
- [x] 仓储接口
  - [x] 在 `repository::user_repository` 定义 trait 接口
  - [x] 实现 SQLx 版仓储：按用户名查询、创建用户、查询资料
  - [x] 补充错误转换与日志
- [x] 仓储测试
  - [x] 使用 `sqlx::test` 编写成功路径测试
  - [x] 编写重复用户名等失败路径测试
  - [x] 根据需要准备测试数据清理流程

## 3. 服务层实现

- [x] 工具准备
  - [x] `util::password`：封装哈希/校验与策略
  - [x] `util::token`：封装签发、验证、Claims 构造
  - [x] 为工具函数编写单元测试
- [x] AuthService 功能
  - [x] 注册：DTO 校验 → 仓储写入 → 事件发布
  - [x] 登录：查库 → 密码比对 → 令牌返回 → 事件发布
  - [x] Profile：依据用户 ID 加载资料
  - [x] Refresh：验证刷新令牌 → 生成新访问令牌
- [x] 服务层测试
  - [x] 覆盖正常流程断言返回值
  - [x] 覆盖业务错误（无效凭证、用户名占用等）
  - [x] 覆盖配置异常（刷新未开启等）

## 4. 接口与中间件

- [x] DTO 与校验
  - [x] 在 `dto` 模块添加请求/响应结构体
  - [x] 使用 `validator` 标注字段规则
  - [x] 为 DTO 编写边界测试（可选）
- [x] 控制器
  - [x] 在 `controller` 编写 `/register`、`/login`、`/refresh`、`/profile` handler
  - [x] 封装统一响应与错误处理
  - [x] 将 `AuthService` 注入 Actix 应用状态
- [ ] 中间件
  - [ ] 实现 `AuthGuard`（解析 Bearer Token）
  - [ ] 将用户信息放入请求扩展
  - [ ] 更新 `main.rs` 中间件链路与路由注册

## 5. 测试与验证

- [ ] 集成测试
  - [ ] 使用 `actix_web::test` 覆盖注册 → 登录 →Profile 流程
  - [ ] 覆盖刷新令牌（启用场景）与未授权访问
  - [ ] 验证统一响应结构与业务码
- [ ] 静态检查
  - [ ] `cargo fmt --check`
  - [ ] `cargo clippy -- -D warnings`
  - [ ] `cargo test --workspace`
  - [ ] `cargo deny` 与 `cargo audit`
- [ ] 性能与可观测性验证
  - [ ] 检查日志是否包含 `request_id`、`user_id`
  - [ ] 验证慢请求告警逻辑
  - [ ] 评估密码哈希成本与吞吐

## 6. 文档与交付

- [ ] API 文档
  - [ ] 更新 `docs/api.md` 增加认证端点
  - [ ] 同步 OpenAPI 描述（含示例与错误码）
  - [ ] 若使用 Postman/Insomnia，更新示例集合
- [ ] 运行手册
  - [ ] 在 `docs/runbook.md` 记录常见故障排查
  - [ ] 说明刷新策略、密钥轮换注意事项
- [ ] PR 交付
  - [ ] 汇总变更与影响范围
  - [ ] 补充测试结果截图或命令输出
  - [ ] 列出潜在风险与后续待办
