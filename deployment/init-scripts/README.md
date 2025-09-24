# 数据库初始化脚本

该目录包含用于初始化数据库结构的脚本。这些脚本需要手动执行或通过应用程序执行。

## 脚本说明

- [01-init-postgres.sh](01-init-postgres.sh): 创建WordMesh应用所需的PostgreSQL数据库表结构
- [02-init-neo4j.sh](02-init-neo4j.sh): 创建WordMesh应用所需的Neo4j数据库约束和索引
- [init-all-databases.sh](init-all-databases.sh): 统一初始化所有数据库的脚本

## 执行方式

脚本现在不会在数据库容器启动时自动执行，需要通过以下方式手动执行：

### 统一初始化（推荐）

```bash
cd deployment/init-scripts
chmod +x ./init-all-databases.sh
./init-all-databases.sh
```

### PostgreSQL初始化

1. 直接运行脚本：
   ```bash
   cd deployment/init-scripts
   chmod +x ./01-init-postgres.sh
   ./01-init-postgres.sh
   ```

2. 或者进入PostgreSQL容器执行：
   ```bash
   docker exec -it wordmesh-postgres bash
   /init-scripts/01-init-postgres.sh
   ```

### Neo4j初始化

1. 直接运行脚本：
   ```bash
   cd deployment/init-scripts
   chmod +x ./02-init-neo4j.sh
   ./02-init-neo4j.sh
   ```

2. 然后执行Cypher脚本：
   ```bash
   docker exec wordmesh-neo4j cypher-shell -u neo4j -p wordmesh123 --file /tmp/neo4j-init.cypher
   ```

3. 或者进入Neo4j容器执行：
   ```bash
   docker cp deployment/init-scripts/02-init-neo4j.sh wordmesh-neo4j:/tmp/
   docker exec -it wordmesh-neo4j bash
   bash /tmp/02-init-neo4j.sh
   ```

## 注意事项

1. 这些脚本需要手动执行
2. PostgreSQL脚本使用psql命令执行SQL语句
3. Neo4j脚本生成Cypher文件，需要通过cypher-shell执行
4. 执行前请确保数据库容器正在运行
5. 执行脚本前请确保脚本具有可执行权限 (chmod +x script.sh)