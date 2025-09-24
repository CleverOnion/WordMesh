#!/bin/bash

# Neo4j数据库初始化脚本
# 用于创建WordMesh应用所需的约束和索引

set -e

echo "开始初始化Neo4j数据库..."

# 生成Cypher脚本文件
cat > /tmp/neo4j-init.cypher << 'EOF'
// 创建约束
// 为Word节点的word属性创建唯一约束
CREATE CONSTRAINT unique_word IF NOT EXISTS FOR (w:Word) REQUIRE w.word IS UNIQUE;

// 为WordRelation节点的relation_type属性创建索引
CREATE INDEX word_relation_type_index IF NOT EXISTS FOR (wr:WordRelation) ON (wr.relation_type);

// 为User节点的username属性创建唯一约束
CREATE CONSTRAINT unique_username IF NOT EXISTS FOR (u:User) REQUIRE u.username IS UNIQUE;

// 为User节点的email属性创建唯一约束
CREATE CONSTRAINT unique_email IF NOT EXISTS FOR (u:User) REQUIRE u.email IS UNIQUE;

// 创建一些基本的词汇关系类型索引
CREATE INDEX word_pos_index IF NOT EXISTS FOR (w:Word) ON (w.part_of_speech);

// 创建示例数据 (可选)
// 创建一些示例单词节点
MERGE (w1:Word {word: "happy", definition: "feeling or showing pleasure or contentment", part_of_speech: "adjective"})
MERGE (w2:Word {word: "joy", definition: "a feeling of great pleasure and happiness", part_of_speech: "noun"})
MERGE (w3:Word {word: "glad", definition: "pleased or happy about something", part_of_speech: "adjective"})

// 创建关系
MERGE (w1)-[:SYNONYM]->(w2)
MERGE (w1)-[:SYNONYM]->(w3)
MERGE (w3)-[:SYNONYM]->(w1)

// 创建词根关系示例
MERGE (w4:Word {word: "unhappy", definition: "not happy; sad", part_of_speech: "adjective"})
MERGE (w1)-[:ANTONYM]->(w4)
MERGE (w4)-[:PREFIX]->(:WordRoot {root: "un"})
MERGE (w4)-[:ROOT]->(w1)

RETURN "Neo4j数据库初始化完成";
EOF

echo "Cypher脚本已生成: /tmp/neo4j-init.cypher"
echo "请使用以下命令执行初始化:"
echo "cypher-shell -u neo4j -p wordmesh123 --file /tmp/neo4j-init.cypher"