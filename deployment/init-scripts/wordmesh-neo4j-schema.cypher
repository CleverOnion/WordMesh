// 创建约束
// 为Word节点的word_id属性创建唯一约束
CREATE CONSTRAINT unique_word_id IF NOT EXISTS FOR (w:Word) REQUIRE w.word_id IS UNIQUE;

// 为Sense节点的sense_id属性创建唯一约束
CREATE CONSTRAINT unique_sense_id IF NOT EXISTS FOR (s:Sense) REQUIRE s.sense_id IS UNIQUE;

// 为Sense节点的user_id属性创建索引
CREATE INDEX sense_user_id_index IF NOT EXISTS FOR (s:Sense) ON (s.user_id);

// 为LINKED_WORD关系的user_id属性创建索引
CREATE INDEX linked_word_user_id_index IF NOT EXISTS FOR ()-[r:LINKED_WORD]-() ON (r.user_id);

// 创建示例数据 (可选)
// 创建一些示例单词节点
MERGE (w1:Word {word_id: 1})
MERGE (w2:Word {word_id: 2})
MERGE (w3:Word {word_id: 3})

// 创建示例义项节点
MERGE (s1:Sense {sense_id: 1, user_id: 101})
MERGE (s2:Sense {sense_id: 2, user_id: 101})
MERGE (s3:Sense {sense_id: 3, user_id: 101})

// 创建关系示例
// Sense -> Word (:DEFINES关系)
MERGE (s1)-[:DEFINES]->(w1)
MERGE (s2)-[:DEFINES]->(w2)

// Sense -> Sense (:RELATED_TO关系)
MERGE (s1)-[:RELATED_TO]->(s2)

// Word -> Word (:LINKED_WORD关系，带user_id属性)
MERGE (w1)-[:LINKED_WORD {user_id: 101}]->(w2)

RETURN "Neo4j数据库初始化完成";