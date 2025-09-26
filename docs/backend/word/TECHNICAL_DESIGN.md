# Word 模块技术设计（TECHNICAL_DESIGN）

本技术设计依据 `docs/backend/word/REQUIREMENTS.md`，说明数据建模、接口契约、核心规则与约束、观测性与测试方案，并与后端统一规范对齐（配置、错误处理、日志与指标、CI 校验）。

## 1. 数据建模

采用 PostgreSQL 存储核心实体（Word/UserWord/UserSense）作为事实来源；采用 Neo4j 存储“关联（Associations）”作为事实来源（SoT）。所有 PostgreSQL DDL 通过 SQLx 迁移管理；Neo4j 通过 Cypher 维护约束与读写。

### 1.1 规范化规则（Canonical Key）

- 文本 → trim → 折叠多空白为单空格 → 去除首尾标点 → tolower → 将内部空格替换为连字符
- 示例：" Graph Database " → "graph-database"

### 1.2 PostgreSQL 表结构（核心实体）

- words（全局词项，无用户字段）

  - id BIGSERIAL PK
  - text VARCHAR(128) NOT NULL
  - canonical_key VARCHAR(160) NOT NULL UNIQUE
  - created_at TIMESTAMPTZ NOT NULL DEFAULT now()
  - 索引：UNIQUE(canonical_key)

- user_words（UserWord，个人词项）

  - id BIGSERIAL PK
  - user_id BIGINT NOT NULL REFERENCES users(id)
  - word_id BIGINT NOT NULL REFERENCES words(id)
  - tags TEXT[] DEFAULT '{}'
  - note TEXT
  - created_at TIMESTAMPTZ NOT NULL DEFAULT now()
  - 约束：UNIQUE(user_id, word_id)
  - 索引：user_id, word_id, GIN(tags)

- user_senses（UserSense，个人义项）
  - id BIGSERIAL PK
  - user_word_id BIGINT NOT NULL REFERENCES user_words(id) ON DELETE CASCADE
  - text TEXT NOT NULL
  - is_primary BOOLEAN NOT NULL DEFAULT false
  - sort_order INT NOT NULL DEFAULT 0
  - created_at TIMESTAMPTZ NOT NULL DEFAULT now()
  - 约束：UNIQUE(user_word_id, text)
  - 约束：同一 user_word 仅允许一条 is_primary = true（部分唯一索引或触发器）
  - 索引：user_word_id, sort_order

备注：删除 user_words 将先删除其下 user_senses（CASCADE），并触发与 Neo4j 的一致性清理（见 4 节）。

### 1.3 Neo4j 图建模（关联 SoT）

节点（Nodes）

- :Word { word_id: BIGINT UNIQUE }
- :UserSense { sense_id: BIGINT UNIQUE, user_id: BIGINT }

关系（Relationships）

- WORD_TO_WORD（词-词，非语义）
  - 形态：(:Word)-[:WORD_TO_WORD {user_id, kind:"similar_form|root_affix", note?, created_at}]-(:Word)
  - 对称：展示层去重；写入时按 (min(word_id), max(word_id)) 固定方向，应用 MERGE 去重
- SENSE_TO_WORD（义 → 词，语义）
  - 形态：(:UserSense {sense_id})-[:SENSE_TO_WORD {user_id, kind:"synonym|antonym|related", note?, created_at}]->(:Word {word_id})

约束与索引（Neo4j 5）

- CREATE CONSTRAINT word_id_unique IF NOT EXISTS FOR (w:Word) REQUIRE w.word_id IS UNIQUE;
- CREATE CONSTRAINT sense_id_unique IF NOT EXISTS FOR (s:UserSense) REQUIRE s.sense_id IS UNIQUE;
- CREATE INDEX word_to_word_idx IF NOT EXISTS FOR ()-[r:WORD_TO_WORD]-() ON (r.user_id, r.kind);
- CREATE INDEX sense_to_word_idx IF NOT EXISTS FOR ()-[r:SENSE_TO_WORD]-() ON (r.user_id, r.kind);

写入（幂等 MERGE）

- 词-词：
  - MERGE (wa:Word {word_id:$minId})
  - MERGE (wb:Word {word_id:$maxId})
  - MERGE (wa)-[r:WORD_TO_WORD {user_id:$uid, kind:$kind}]->(wb)
  - ON CREATE SET r.created_at=datetime(), r.note=$note
  - ON MATCH SET r.note=coalesce($note, r.note)
- 义 → 词：
  - MATCH (s:UserSense {sense_id:$sid})
  - MATCH (w:Word {word_id:$wid})
  - MERGE (s)-[r:SENSE_TO_WORD {user_id:$uid, kind:$kind}]->(w)
  - ON CREATE SET r.created_at=datetime(), r.note=$note
  - ON MATCH SET r.note=coalesce($note, r.note)

清理（级联）

- 删除 UserSense（PG）后：
  - MATCH (s:UserSense {sense_id:$sid})-[r:SENSE_TO_WORD]->() DELETE r;
  - 可选：DETACH DELETE s（若无需保留缓存节点）

## 2. 领域与服务

- CanonicalService：text → canonical_key（纯函数，单测覆盖边界）
- WordService：
  - add_to_my_network(text, tags?, note?, first_sense?) → 复用/创建 words；UPSERT user_words；可选新增首条 user_senses
  - remove_from_my_network(user_word_id) → 删除 user_words；触发 Neo4j 清理（该词项下义项对应的关系）
- SenseService：
  - add_sense(user_word_id, text, is_primary?)
  - update_sense(sense_id, text?, is_primary?, sort_order?)
  - remove_sense(sense_id) → PG 删除后调用 Neo4j 清理该 sense 的 SENSE_TO_WORD
- AssocService（Neo4j）：
  - create_word_link(word_id_a, word_id_b, kind) → 端点排序 + MERGE WORD_TO_WORD；自链接/去重校验
  - create_sense_word_link(sense_id, target_word_id, kind) → 目标词未在词网则 PG UPSERT user_words，再 MERGE SENSE_TO_WORD；禁止将源义项所属词作为目标
  - list_links_by_word(word_id, kind?) → 查询相邻 WORD_TO_WORD
  - list_links_by_sense(sense_id, kind?) → 查询相邻 SENSE_TO_WORD
  - delete_link(...) → 依据端点与属性删除关系（或先查出关系再删）

服务统一返回 Result<T, AppError>。

## 3. API 设计（统一响应）

前缀：`/api/v1/words`

- POST `/my` 加入我的词网
- POST `/my/{user_word_id}/senses` 新增个人义项
- PATCH `/my/senses/{sense_id}` 更新个人义项
- DELETE `/my/senses/{sense_id}` 删除个人义项（级联清理 Neo4j 义 → 词关系）
- DELETE `/my/{user_word_id}` 从我的词网移除（级联清理其下义项与关联）
- GET `/my/search?q=...&scope=word|sense|both&limit&offset` 搜索
- POST `/associations/word` 创建 UserWordLink（kind: similar_form|root_affix）→ Neo4j
- POST `/associations/sense-word` 创建 UserSenseWordLink（kind: synonym|antonym|related）→ Neo4j（并确保 PG user_words 存在）
- GET `/associations?endpoint_type=word|sense&endpoint_id&kind&limit&offset` 列表/筛选 → Neo4j
- DELETE `/associations/{id}` 删除关联 → Neo4j（或通过端点+属性定位）

响应体：`{ code, message, data, traceId, timestamp }`。

## 4. 校验、幂等与一致性

- 规范化：创建/搜索/比较统一使用 CanonicalService
- 幂等：
  - user_words：UNIQUE(user_id, word_id) + ON CONFLICT DO NOTHING
  - Neo4j：MERGE 去重；词-词端点先排序（min/max）保证对称幂等
- 主义项唯一：部分唯一索引（user_word_id, is_primary WHERE is_primary=true）或触发器
- 自链接与约束：
  - WORD_TO_WORD：word_a != word_b
  - SENSE_TO_WORD：target_word_id != 源义项所属词
- 一致性（PG ↔ Neo4j）：
  - 删除 user_senses 后，同步删除 Neo4j 中该 sense 的 SENSE_TO_WORD 关系（幂等）
  - 删除 user_words 时，先删其下 user_senses（将触发上一步清理），再删 user_word
- 自动加入词网：创建 SENSE_TO_WORD 前，确保 PG 中 user_words(user_id, target_word_id) 存在（UPSERT）

## 5. 错误模型与业务码

- 4001 VALIDATION_FAILED
- 4201 WORD_ALREADY_EXISTS（加入词网幂等命中）
- 4202 WORD_NOT_IN_NETWORK（或自动加入失败）
- 4203 SENSE_DUPLICATE
- 4204 PRIMARY_CONFLICT
- 4301 LINK_EXISTS（Neo4j MERGE 命中视为已存在）
- 4302 LINK_SELF_FORBIDDEN
- 4303 LINK_TARGET_NOT_FOUND
- 4304 LINK_TYPE_INVALID
- 4305 LINK_LIMIT_EXCEEDED

## 6. 观测性与安全

- tracing：记录 request_id、user_id、route、duration；对 Neo4j 操作记录 op 与耗时；>500ms 警告
- metrics：
  - counters：words_added, senses_added, links_created
  - histograms：handler_duration_seconds by route；neo4j_query_duration_seconds by op
  - gauges：user_word_count, user_link_count（可选）
- 安全：字段 validator 校验；SQL 参数化；日志脱敏

## 7. 测试策略

- 单元：规范化函数；义项唯一与排序；自链接与去重逻辑（端点排序）
- 集成：
  - UC1–UC10 链路（加入 → 义项 → 义 → 词/词 → 词关联 → 查询 → 删义项级联 → 删词项）
  - 搜索（词项/义项）与分页
  - 跨存储一致性：PG 删除后 Neo4j 关系清理验证
- 属性：随机文本验证规范化稳定性

## 8. 迁移与演进

- PostgreSQL：初始迁移创建 words、user_words、user_senses 及索引与约束
- Neo4j：部署时执行约束与索引的 Cypher；按需维护升级脚本
- 后续可扩展更多关联类型与可视化，保持 PG 为核心实体 SoT、Neo4j 为关联 SoT
