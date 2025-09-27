#!/bin/bash

# Neo4j数据库初始化脚本
# 用于创建WordMesh应用所需的约束和索引

set -e

echo "开始初始化Neo4j数据库..."

# 生成Cypher脚本文件
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CYTHER_FILE="/tmp/wordmesh-neo4j-schema.cypher"
cat "$SCRIPT_DIR/wordmesh-neo4j-schema.cypher" > "$CYTHER_FILE"

echo "Cypher脚本已生成: $CYTHER_FILE"
echo "请使用以下命令执行初始化:"
echo "cypher-shell -u neo4j -p wordmesh123 --file $CYTHER_FILE"