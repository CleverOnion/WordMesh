# 鉴权模块交付说明（Docs & Delivery）

## 本次交付内容

- 新增接口文档：`docs/api.md`（鉴权端点、统一响应、错误码、示例 cURL）
- 新增 OpenAPI 规范：`docs/openapi.yaml`（Health + Auth）
- 新增运行手册：`docs/RUNBOOK.md`（启动步骤、排障、运维要点）
- 更新开发任务规划：勾选“任务 6. 文档与交付”项（见 `docs/backend/auth/IMPLEMENTATION_PLAN.md`）

## 变更影响范围

- 文档新增，不影响现有接口行为；作为 API 消费方与运维使用的权威资料。

## 验收与测试

- 控制器与服务层单元/集成测试已覆盖：注册/登录/刷新/资料、未授权访问、错误码映射。
- 手动验证：
  - `GET /api/v1/health` 返回统一成功包
  - 鉴权全链路：register → login → profile → refresh

## 风险与注意事项

- 配置文件中仍存在示例密钥与口令，生产环境务必通过环境变量覆盖（参见 RUNBOOK）。
- 统一响应规范与早期文档的 `{ data, meta, error }` 描述存在差异，需后续统一。
- 未接入 OpenAPI 文档在线路由（仅规范文件），可后续新增 `/api/docs` 或基于 Swagger UI 静态托管。

## 后续待办（建议）

- 接入 JSON 日志、慢请求告警与 Prometheus 指标端点。
- CI 增加 `fmt`/`clippy -D warnings`/`test --workspace`/`cargo deny`/`cargo audit`。
- 将敏感配置迁移至环境变量并从 repo 移除示例秘钥。
