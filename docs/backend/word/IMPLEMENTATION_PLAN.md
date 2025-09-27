# Word 模块开发任务规划

本计划对齐 `docs/backend/word/REQUIREMENTS.md` 与 `docs/backend/word/TECHNICAL_DESIGN.md`，在鉴权模块风格基础上增加细化项，以确保工程可落地、可验证、可观测。

## 0. 范围与目标（摘要）

- 全局词项（Word，PG）无用户字段；以规范化键全局唯一
- 个人词项（UserWord，PG）：用户与词的纳入关系，含标签/备注
- 个人义项（UserSense，PG）：同一词下可多条，支持主义项与排序；删义项需级联清理义 → 词关联
- 关联（Neo4j 为 SoT）：
  - 词-词（WORD_TO_WORD）：similar_form、root_affix（对称）
  - 义 → 词（SENSE_TO_WORD）：synonym、antonym、related（目标词必须不同于源义项所属词；若不在词网自动加入）

---

## 1. 准备阶段

- [x] 同步文档
  - [x] 阅读并对齐 `REQUIREMENTS.md`、`TECHNICAL_DESIGN.md`
    - 已核对需求、技术设计与实施计划三份文档，确认目标、表结构、服务接口及观测性/测试策略一致。
  - [x] 整理开放问题并形成 ADR 草案（canonical 规则细节、主义项唯一实现、搜索排序规则）
    - 当前三项规则在技术设计中已有明确方案，暂无新增开放问题，ADR 草案待后续实现阶段补充细节说明。
- [x] 开发环境与配置
  - [x] 在 `config/settings.rs` 增加：
    - [x] `postgres`（url、pool_size、connect_timeout）
    - [x] `neo4j`（url、user、password、pool_size、query_timeout）
    - `DatabaseSettings` 与 `Neo4jSettings` 均已具备连接信息、池参数与超时；`Settings::validate()` 会调用对应校验方法。
  - [x] `Settings::validate()` 完整校验；更新 `config/*.toml` 占位键
    - `database.pool_size`、`neo4j.pool_size`、`neo4j.query_timeout_seconds` 已纳入配置，并在 `config/*.toml` 提供默认值；校验逻辑确保数值有效。
  - [x] `deployment/docker-compose.yml` 校验本地 PG/Neo4j 启停与初始化顺序
    - Compose 文件提供 Postgres(5432) 与 Neo4j(7687/7474) 服务，容器命名与初始化脚本保持一致；`deployment/init-scripts/` 中的统一脚本已验证可按 README 指引顺序启动并执行初始化。
- [x] 错误体系统一
  - [x] 扩展 `util::error` 新增 `BusinessError::{Word,Link}`
    - 已新增 `WordError`、`LinkError` 并纳入 `BusinessError` 与统一响应处理。
  - [x] 定义业务码映射（WORD*ALREADY_EXISTS、SENSE_DUPLICATE、PRIMARY_CONFLICT、LINK* 等）
    - `WordError` 映射 4201~4204，`LinkError` 映射 4301~4305，对齐需求中的错误码规划。
  - [x] 补充错误单元测试（验证统一响应 `{ code,message,data,traceId,timestamp }`）
    - 新增 `WordError`、`LinkError` 对应的响应测试，覆盖业务码与字段结构。

## 2. 数据访问与领域模型

- [ ] 领域建模
  - [ ] 值对象：`CanonicalKey`（text → canonical_key）
  - [ ] 实体/聚合：`UserWord`、`UserSense`（不变式：唯一性、主义项唯一、排序）
- [ ] PostgreSQL 迁移（SQLx）
  - [ ] `words`：id、text、canonical_key UNIQUE、created_at
  - [ ] `user_words`：id、user_id、word_id、tags、note、UNIQUE(user_id,word_id)
  - [ ] `user_senses`：id、user_word_id(FK ON DELETE CASCADE)、text、is_primary、sort_order、UNIQUE(user_word_id,text)
  - [ ] 主义项唯一：部分唯一索引（user_word_id, is_primary WHERE is_primary=true）或触发器（二选一并记录 ADR）
  - [ ] 索引：`GIN(tags)`、`user_word_id`、`sort_order`
- [ ] Neo4j 初始化（Cypher）
  - [ ] 约束：`(w:Word) word_id UNIQUE`、`(s:UserSense) sense_id UNIQUE`
  - [ ] 索引：WORD_TO_WORD(r.user_id,r.kind)、SENSE_TO_WORD(r.user_id,r.kind)
  - [ ] 提供 `deployment/init-scripts/*.cypher` 与执行说明
- [ ] 仓储接口
  - [ ] `repository::word`（PG）：words/user_words/user_senses CRUD + 查询 + 搜索
  - [ ] `repository::graph`（Neo4j）：词-词与义 → 词关联的查询与写入接口
- [ ] 仓储测试
  - [ ] SQLx 基础用例（唯一约束/UPSERT/CASCADE）
  - [ ] Neo4j 最小链路（MERGE 幂等、端点排序、筛选）

## 3. 服务层实现

- [ ] 工具准备
  - [ ] `util::canonical`：规范化函数（trim/折叠空白/去首尾标点/小写/空格 → 连字符）
  - [ ] 输入校验辅助（标签 20 个、标签长度 1..24、备注长度上限、文本不可全空白）
- [ ] WordService
  - [ ] `add_to_my_network(text,tags?,note?,first_sense?)`
    - [ ] 生成 canonical_key → 查/建 `words`
    - [ ] UPSERT `user_words`（幂等），可选插入首条 `user_senses`
    - [ ] 返回 UserWord 详情（含个人义项列表）
  - [ ] `remove_from_my_network(user_word_id)`
    - [ ] 先删该词项下的 `user_senses`（触发后续图清理）→ 再删 `user_words`
  - [ ] `search_in_my_network(q,scope,limit,offset)`：词项/义项维度、前缀/包含、分页（默认 20，最大 100）
- [ ] SenseService
  - [ ] `add_sense(user_word_id,text,is_primary?)`：同词项下文本唯一；必要时清空旧主义项
  - [ ] `update_sense(sense_id,text?,is_primary?,sort_order?)`：确保唯一与主义项唯一
  - [ ] `remove_sense(sense_id)`：PG 删除后触发 Neo4j 清理该 sense 的 SENSE_TO_WORD
- [ ] AssocService（Neo4j）
  - [ ] `create_word_link(word_id_a,word_id_b,kind)`：端点排序 + MERGE；禁止自链接；对称去重
  - [ ] `create_sense_word_link(sense_id,target_word_id,kind)`：目标词 ≠ 源词；目标词不在词网则 UPSERT `user_words`；MERGE 去重
  - [ ] `list_links_by_word(word_id,kind?,limit,offset)`
