# WordMesh

WordMesh 是一个个人知识网络构建工具，旨在帮助用户通过连接单词和概念来构建和探索自己的语义世界。

## 我可以用 WordMesh 做什么？

作为一名用户，我将能够：

- **管理我的专属账户**:

  - [x] 我可以注册一个专属账号，并随时登录，以安全地保管我的所有知识。

- **创建我的知识卡片**:

  - [ ] 我可以为一个单词（比如 "run"）添加我自己的理解或翻译（比如“跑”、“运行”或“经营”）。
  - [ ] 我可以为我添加的每个理解或单词本身随时写下笔记和感悟。

- **连接我的知识网络**:

  - [ ] **关联我的不同理解**: 我可以将我创建的两个不同“知识卡片”关联起来。例如，将我对“run”的理解“经营（公司）”与我对“business”的理解“商业活动”联系在一起。
  - [ ] **连接两个独立的单词**: 我可以在两个公开的单词之间建立一条我专属的联想路径。例如，直接将“memory”和“storage”这两个单词连接起来，记录我发现它们之间的相似之处。

- **探索和使用我的知识**:
  - [ ] 我可以轻松地搜索我记录过的所有单词和想法。
  - [ ] (未来) 我将能以图形化的方式直观地看到我的整个知识网络是如何连接的。

## 当前完成情况

项目目前处于**核心功能开发阶段**，身份认证模块已完成并可正常使用。

### 已完成功能

- [x] **产品需求文档 (`PRD.md`)**: 已完成初步的产品需求规划。
- [x] **技术设计文档 (`Technical_Design.md`)**: 已完成详细的后端架构、技术选型和数据库设计。
  - **技术栈**: Rust (Actix Web), PostgreSQL, Neo4j
  - **数据模型**: 已确定关系型和图数据库的最终模型。
- [x] **数据库初始化脚本**: 已完成 PostgreSQL 和 Neo4j 数据库的初始化脚本。
- [x] **项目骨架搭建**: 已完成项目目录结构和基础代码框架。
- [x] **身份认证模块**: 已完成用户注册、登录、JWT 令牌管理、用户资料获取等功能。
  - 用户注册与登录
  - JWT 访问令牌和刷新令牌
  - 密码哈希与验证
  - 认证中间件与路由保护
  - 统一错误处理与响应格式

### 开发中功能

- [ ] **知识卡片管理**: 单词与概念的创建、编辑、删除功能。
- [ ] **知识网络构建**: 卡片之间的关联与连接功能。
- [ ] **搜索与探索**: 知识网络的搜索与可视化功能。

## 如何开始

### 快速启动

1. 启动数据库服务：

```bash
cd deployment
docker compose up -d
```

2. 运行后端服务：

```bash
cd WordMesh-backend
cargo run
```

3. 访问服务：
   - 后端 API: http://localhost:8080
   - API 文档: 参考 [docs/api.md](docs/api.md)
   - Neo4j Browser: http://localhost:7474

### 快速体验认证功能

1. 注册新用户：

```bash
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"demo_user","password":"demo123456"}'
```

2. 用户登录：

```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"demo_user","password":"demo123456"}'
```

3. 获取用户资料（需要先从登录响应中获取 access_token）：

```bash
curl -X GET http://localhost:8080/api/v1/auth/profile \
  -H "Authorization: Bearer <YOUR_ACCESS_TOKEN>"
```

## 📖 文档

详细文档请查看：[文档索引](docs/README.md)
