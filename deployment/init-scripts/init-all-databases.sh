#!/bin/bash

# 统一数据库初始化脚本
# 用于同时初始化PostgreSQL和Neo4j数据库

set -e

echo "====================================="
echo "开始初始化所有数据库..."
echo "====================================="

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "脚本目录: $SCRIPT_DIR"

# 初始化PostgreSQL数据库
echo ""
echo "-------------------------------------"
echo "正在初始化PostgreSQL数据库..."
echo "-------------------------------------"

if [ -f "$SCRIPT_DIR/01-init-postgres.sh" ]; then
    chmod +x "$SCRIPT_DIR/01-init-postgres.sh"
    "$SCRIPT_DIR/01-init-postgres.sh"
    echo "PostgreSQL数据库初始化完成"
else
    echo "错误: 找不到PostgreSQL初始化脚本 $SCRIPT_DIR/01-init-postgres.sh"
    exit 1
fi

# 初始化Neo4j数据库
echo ""
echo "-------------------------------------"
echo "正在初始化Neo4j数据库..."
echo "-------------------------------------"

if [ -f "$SCRIPT_DIR/02-init-neo4j.sh" ]; then
    chmod +x "$SCRIPT_DIR/02-init-neo4j.sh"
    "$SCRIPT_DIR/02-init-neo4j.sh"
    echo "Neo4j数据库初始化脚本执行完成"
    
    # 将Neo4j初始化脚本复制到容器中并执行
    echo "将初始化脚本复制到Neo4j容器..."
    docker cp "$SCRIPT_DIR/02-init-neo4j.sh" wordmesh-neo4j:/tmp/
    docker cp /tmp/neo4j-init.cypher wordmesh-neo4j:/tmp/ 2>/dev/null || echo "注意: /tmp/neo4j-init.cypher 文件不存在，将从容器内生成"
    
    echo "在Neo4j容器中执行初始化..."
    docker exec wordmesh-neo4j bash /tmp/02-init-neo4j.sh
    docker exec wordmesh-neo4j cypher-shell -u neo4j -p wordmesh123 --file /tmp/neo4j-init.cypher
    echo "Neo4j数据库初始化完成"
else
    echo "错误: 找不到Neo4j初始化脚本 $SCRIPT_DIR/02-init-neo4j.sh"
    exit 1
fi

echo ""
echo "====================================="
echo "所有数据库初始化完成!"
echo "====================================="