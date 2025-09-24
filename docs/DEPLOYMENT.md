# WordMesh 部署指南

本指南将介绍如何使用Docker和Docker Compose来部署WordMesh应用及其依赖的服务。

## 目录结构

```
WordMesh/
├── deployment/
│   ├── docker-compose.yml      # Docker服务编排文件
│   └── init-scripts/           # 数据库初始化脚本
│       ├── 01-init-postgres.sh # PostgreSQL初始化脚本
│       ├── 02-init-neo4j.sh    # Neo4j初始化脚本
│       └── init-all-databases.sh # 所有数据库初始化脚本
└── ...
```

## 部署步骤

### 1. 安装依赖

确保您已安装以下软件：
- Docker (v20.10+)
- Docker Compose (v1.29+)

### 2. 克隆项目

```bash
git clone <repository-url>
cd WordMesh
```

### 3. 启动服务

使用Docker Compose一键启动所有服务：

```bash
cd deployment
docker compose up -d
```

这将启动以下服务：
- PostgreSQL数据库 (端口: 5432)
- Neo4j图数据库 (端口: 7474, 7687)

### 4. 初始化数据库

数据库服务启动后，需要初始化数据库结构：

```bash
cd deployment
./init-scripts/init-all-databases.sh
```

或者分别初始化每个数据库：

```bash
# 初始化PostgreSQL
docker exec -it wordmesh-postgres /init-scripts/01-init-postgres.sh

# 初始化Neo4j
docker cp init-scripts/02-init-neo4j.sh wordmesh-neo4j:/tmp/
docker exec wordmesh-neo4j bash /tmp/02-init-neo4j.sh
docker exec wordmesh-neo4j cypher-shell -u neo4j -p wordmesh123 --file /tmp/neo4j-init.cypher
```

### 5. 验证服务状态

检查所有服务是否正常运行：

```bash
cd deployment
docker compose ps
```

### 6. 访问服务

- **PostgreSQL**: 通过 `localhost:5432` 访问
- **Neo4j Browser**: 通过 `http://localhost:7474` 访问

### 7. 停止服务

```bash
cd deployment
docker compose down
```

如果要同时删除数据卷（注意：这将删除所有数据）：

```bash
cd deployment
docker compose down -v
```

## 环境变量配置

环境变量在以下文件中配置：

1. `docker-compose.yml` - Docker服务环境变量
2. `WordMesh-backend/.env` - 后端应用环境变量

## 数据持久化

数据将持久化存储在Docker卷中：
- `postgres_data` - PostgreSQL数据
- `neo4j_data` - Neo4j数据
- `neo4j_logs` - Neo4j日志

## 故障排除

### 查看日志

```bash
# 查看所有服务日志
cd deployment
docker compose logs

# 查看特定服务日志
docker compose logs postgres
docker compose logs neo4j
```

### 重新构建镜像

```bash
cd deployment
docker compose build
```

### 强制重新创建容器

```bash
cd deployment
docker compose up -d --force-recreate
```