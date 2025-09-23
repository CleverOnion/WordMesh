# 数据库初始化脚本

该目录包含用于初始化数据库结构的脚本。这些脚本需要手动执行或通过应用程序执行。

## 脚本说明

- [01-init-postgres.sh](01-init-postgres.sh): 创建WordMesh应用所需的数据库表结构

## 执行方式

脚本现在不会在PostgreSQL容器启动时自动执行，需要通过以下方式手动执行：

1. 进入PostgreSQL容器：`docker exec -it wordmesh-postgres bash`
2. 执行脚本：`/init-scripts/01-init-postgres.sh`

或者通过应用程序在启动时执行数据库初始化。

## 注意事项

1. 这些脚本需要手动执行
2. 脚本使用PostgreSQL的psql命令执行SQL语句