# WordMesh Backend

WordMesh 后端服务，使用 Rust + Actix Web + PostgreSQL + Neo4j 构建。

## 技术栈

- **编程语言**: Rust (Edition 2021)
- **Web 框架**: Actix Web 4.4
- **关系型数据库**: PostgreSQL 16
- **图数据库**: Neo4j 5.15
- **数据库驱动**: SQLx (PostgreSQL), neo4rs (Neo4j)
- **序列化**: Serde + Serde JSON
- **配置管理**: dotenv + config
- **日志**: tracing + tracing-subscriber
- **认证**: JWT + bcrypt

## 项目结构

```
src/
├── main.rs              # 程序入口
├── config/              # 配置管理
│   ├── mod.rs
│   └── settings.rs
├── controller/          # API 接口层
├── application/         # 应用服务层
├── service/             # 领域服务层
├── repository/          # 仓储层
├── domain/              # 领域模型层
│   ├── mod.rs
│   ├── user.rs
│   ├── word.rs
│   ├── sense.rs
│   └── note.rs
├── event/               # 事件处理
├── middleware/          # 中间件
├── dto/                 # 数据传输对象
└── util/                # 通用工具
    ├── mod.rs
    ├── error.rs
    └── password.rs
```

## 快速开始

### 1. 环境要求

- Rust 1.70+
- PostgreSQL 16+
- Neo4j 5.15+

### 2. 启动数据库服务

```bash
cd ../deployment
docker compose up -d
```

### 3. 运行后端服务

```bash
# 开发环境 (默认)
cargo run

# 或指定环境
RUST_ENV=development cargo run
RUST_ENV=testing cargo run
RUST_ENV=production cargo run
```

### 4. 健康检查

```bash
curl http://127.0.0.1:8080/api/v1/health
```

## 开发

### 构建

```bash
cargo build
```

### 运行测试

```bash
cargo test
```

### 代码检查

```bash
cargo check
cargo clippy
cargo fmt
```

## API 端点

- `GET /api/v1/health` - 健康检查

更多 API 端点将在后续开发中添加。

## 依赖版本

所有依赖都使用最新稳定版本：

- actix-web: 4.4
- sqlx: 0.7
- neo4rs: 0.19
- serde: 1.0
- tokio: 1.35
- tracing: 0.1
- 等等...

## 许可证

本项目采用 MIT 许可证。
