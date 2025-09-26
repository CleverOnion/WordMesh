# WordMesh 后端运行手册（Runbook）

本手册面向开发与运维，涵盖启动步骤、常见故障定位与运维注意事项。

## 启动步骤

1. 启动依赖服务（PostgreSQL、Neo4j）：

```bash
cd deployment && docker compose up -d
```

2. 初始化数据库（可选，首次或需要重置时）：

```bash
cd deployment && ./init-scripts/init-all-databases.sh
```

3. 运行后端：

```bash
cd WordMesh-backend
# 默认 development 环境
cargo run
# 或指定环境
RUST_ENV=testing cargo run
RUST_ENV=production cargo run
```

4. 健康检查：

```bash
curl http://127.0.0.1:8080/api/v1/health
```

## 配置与敏感信息

- 配置文件位于 `WordMesh-backend/config/*.toml`，应通过环境变量 `WORDMESH_*` 覆盖敏感信息（数据库口令、JWT 秘钥等）。
- 认证配置：
  - `auth.jwt.algorithm`: `HS256` 或 `RS256`
  - `auth.jwt.secret`: HS256 下必填
  - `auth.jwt.private_key` / `public_key`: RS256 下必填（PEM）
  - `auth.jwt.refresh_ttl_secs = 0` 表示关闭刷新功能

## 日志与追踪

- 所有响应包含 `traceId`；同时中间件会在响应头输出 `X-Request-Id`，用于日志关联。
- 可通过设置 `RUST_LOG` 或配置中的日志等级来调整日志输出。

## 常见问题排查

1. 无法连接数据库

   - 现象：启动时报 `database connection failed` 或 SQLx 报连接错误
   - 检查：
     - Docker 容器状态：`docker compose ps`
     - 端口冲突：确保 `5432` 未被占用
     - 配置：`WORDMESH_DATABASE_*` 环境变量是否正确

2. 登录总是返回 `Invalid credentials`

   - 现象：`/auth/login` 返回 `code=4011`
   - 排查：
     - 用户是否已通过 `/auth/register` 成功创建
     - 数据库中 `users` 表是否存在目标用户名
     - 密码是否正确（bcrypt 校验）

3. 刷新失败或返回 `Refresh token disabled`

   - 现象：`/auth/refresh` 返回 `code=4014` 或无 `refresh_token`
   - 排查：
     - 检查配置项 `auth.jwt.refresh_ttl_secs` 是否为 0（代表关闭）
     - 检查 `refresh_token` 是否过期或格式不正确

4. `Token expired` / `Token invalid`

   - 现象：访问受保护接口返回 `4012/4013`
   - 排查：
     - 访问令牌是否过期（对比 `iat/exp`）
     - 发行算法与密钥是否与服务端一致（HS256/RS256、秘钥/公私钥）

5. Neo4j 初始化失败
   - 现象：Cypher 约束/索引创建失败
   - 排查：
     - 通过 Neo4j 浏览器或 `cypher-shell` 重试 `deployment/init-scripts/wordmesh-neo4j-schema.cypher`
     - 检查 `bolt://localhost:7687` 连通性与认证信息

## 运维注意事项

- 密钥轮换：
  - HS256：滚动发布前下发新 `secret`，并短期内接受双秘钥（需要代码支持 KID/多 Key）。
  - RS256：准备新密钥对，发布后切换 KID 并逐步失效旧公钥。
- 密码哈希成本：
  - `auth.password.min_length` 仅影响校验；哈希成本由 bcrypt cost（当前按最小长度推导）决定，建议评估在生产环境的吞吐与延迟，并固定 cost。
- 备份与恢复：
  - PostgreSQL：定期 base backup + WAL；
  - Neo4j：备份 data 目录与导出图快照。

## 验证与回滚

- 构建与测试：

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test --workspace
```

- 出现问题回滚：
  - 使用上一可用版本镜像/二进制；
  - 数据库变更需配套回滚脚本。
