# WordMesh 配置管理

## 概述

WordMesh 使用简单的多环境配置管理，支持开发、测试和生产环境。

## 配置文件

```
WordMesh-backend/config/
├── default.toml          # 默认配置
├── development.toml      # 开发环境配置
├── testing.toml          # 测试环境配置
└── production.toml       # 生产环境配置
```

## 如何指定环境

### 方法 1: 环境变量 (推荐)
```bash
# 开发环境 (默认)
RUST_ENV=development cargo run

# 测试环境
RUST_ENV=testing cargo run

# 生产环境
RUST_ENV=production cargo run
```

### 方法 2: 直接运行 (使用默认开发环境)
```bash
cargo run  # 自动使用 development 环境
```

## 配置优先级

配置加载顺序（后面的会覆盖前面的）：

1. **默认配置** - `config/default.toml`
2. **环境配置** - `config/{environment}.toml`
3. **环境变量** - `WORDMESH_*` 前缀

## 环境变量覆盖

如果需要覆盖配置文件中的设置，可以使用环境变量：

```bash
# 覆盖数据库配置
export WORDMESH_DATABASE_HOST=remote-db.com
export WORDMESH_DATABASE_PORT=5433

# 覆盖应用端口
export WORDMESH_APPLICATION_PORT=9000

# 然后运行
RUST_ENV=production cargo run
```

## 配置示例

查看各环境的配置文件：
- `config/default.toml` - 基础配置
- `config/development.toml` - 开发环境
- `config/testing.toml` - 测试环境  
- `config/production.toml` - 生产环境
