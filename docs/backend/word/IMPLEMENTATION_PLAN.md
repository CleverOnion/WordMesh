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

- [ ] 同步文档
  - [ ] 阅读并对齐 `REQUIREMENTS.md`、`TECHNICAL_DESIGN.md`
  - [ ] 整理开放问题并形成 ADR 草案（canonical 规则细节、主义项唯一实现、搜索排序规则）
- [ ] 开发环境与配置
  - [ ] 在 `config/settings.rs` 增加：
    - [ ] `postgres`（url、pool_size、connect_timeout）
    - [ ] `neo4j`（url、user、password、pool_size、query_timeout）
  - [ ] `Settings::validate()` 完整校验；更新 `config/*.toml` 占位键
  - [ ] `deployment/docker-compose.yml` 校验本地 PG/Neo4j 启停与初始化顺序
- [ ] 错误体系统一
  - [ ] 扩展 `util::error` 新增 `BusinessError::{Word,Link}`
  - [ ] 定义业务码映射（WORD*ALREADY_EXISTS、SENSE_DUPLICATE、PRIMARY_CONFLICT、LINK*\* 等）
  - [ ] 补充错误单元测试（验证统一响应 `{ code,message,data,traceId,timestamp }`）

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
  - [ ] `list_links_by_word(word_id,kind?,limit,offset)`；`list_links_by_sense(sense_id,kind?,limit,offset)`
  - [ ] `delete_link(...)`：依据端点+属性或关系 id 删除
- [ ] 服务层测试
  - [ ] 覆盖正常与错误路径：重复加入、义项重复、主义项冲突、自链接、上限、目标词自动加入失败等

## 4. 接口与中间件

- [ ] DTO 与校验（validator）
  - [ ] Word：加入、搜索、UserWord 详情
  - [ ] Sense：新增/更新/删除
  - [ ] Associations：创建/列表/删除（词-词、义 → 词）
  - [ ] 统一分页参数（limit<=100，offset<=10000）与字段级错误返回
- [ ] 控制器与路由
  - [ ] `/api/v1/words/my`（POST）加入我的词网
  - [ ] `/api/v1/words/my/{user_word_id}/senses`（POST）新增个人义项
  - [ ] `/api/v1/words/my/senses/{sense_id}`（PATCH/DELETE）更新/删除个人义项
  - [ ] `/api/v1/words/my/search`（GET）搜索
  - [ ] `/api/v1/words/associations/word`（POST）创建词-词关联（Neo4j）
  - [ ] `/api/v1/words/associations/sense-word`（POST）创建义 → 词关联（Neo4j）
  - [ ] `/api/v1/words/associations`（GET）列表/筛选（Neo4j）
  - [ ] `/api/v1/words/associations/{id}`（DELETE）删除关联（Neo4j 或按端点删除）
  - [ ] 统一响应包装与错误映射（WORD*\*、SENSE*\_、LINK\_\_）
- [ ] 中间件
  - [ ] `RequestId`、`AuthGuard`、慢请求告警（>500ms warn）
  - [ ] 在 `main.rs` 注入 PG/Neo4j 连接池与服务实例

## 5. 观测性与性能

- [ ] tracing
  - [ ] 每个 handler 记录 `request_id`、`user_id`、route、duration；关键分支打点（UPSERT 命中/创建、新增义项、MERGE 命中）
- [ ] metrics（Prometheus）
  - [ ] counters：`word_words_added_total`、`word_senses_added_total`、`word_links_created_total`
  - [ ] histograms：`word_handler_duration_seconds` 按 route；`word_neo4j_query_duration_seconds` 按 op
  - [ ] gauges（可选）：`word_user_word_count`、`word_user_link_count`
- [ ] 性能目标（本地 Debug 参考）
  - [ ] 加入词网 P95 < 80ms（PG 命中）/ < 150ms（PG 新建 + 义项）
  - [ ] 创建关联 P95 < 120ms（含 Neo4j MERGE）
  - [ ] 搜索 20 条结果 P95 < 120ms

## 6. 安全与合规

- [ ] 仅登录用户可用；所有输入均 `validator` 校验
- [ ] SQL 参数化、Cypher 参数绑定，避免注入
- [ ] 日志脱敏（文本过长截断、移除凭证信息）
- [ ] 依赖治理：`cargo audit`、`cargo deny` 无阻断风险

## 7. 测试与验证

- [ ] 单元测试
  - [ ] 规范化函数性质与边界；标签/备注规则；主义项唯一
  - [ ] Neo4j 端点排序与去重、自链接禁止
- [ ] 集成测试（UC1–UC10）
  - [ ] UC1 加入词网（幂等命中）
  - [ ] UC2 新增义项；UC3 更新/排序/主义项；UC4 删除并级联清理
  - [ ] UC5 移出词网（清理个人数据与关联）
  - [ ] UC6 搜索（词项/义项、分页）
  - [ ] UC7–UC10 关联创建/筛选/删除（词-词、义 → 词）
- [ ] 合同测试
  - [ ] `docs/openapi.yaml` 对齐响应结构与错误码；示例覆盖
- [ ] 静态检查（CI）
  - [ ] `cargo fmt --check`
  - [ ] `cargo clippy -- -D warnings`
  - [ ] `cargo test --workspace`
  - [ ] `cargo deny`、`cargo audit`

## 8. 文档与交付

- [ ] API 文档：更新 `docs/api.md`、`docs/openapi.yaml`（新增端点与错误码）
- [ ] 运行手册：`docs/runbook.md` 增加 PG/Neo4j 初始化、排错与数据清理指引
- [ ] 交付说明：`docs/DELIVERY.md` 汇总变更、测试结果与风险

## 9. 验收标准（Checklist）

- [ ] 规范化键生成一致；同词去重生效；加入词网幂等
- [ ] 同词项下义项文本不可重复；主义项唯一；删除义项级联清理义 → 词关联
- [ ] 词-词/义 → 词关联创建去重、自链接禁止；创建义 → 词可自动将目标词加入我的词网
- [ ] 搜索支持词项/义项、前缀/包含、分页；默认 20，最大 100
- [ ] 统一响应/错误码正确；tracing/metrics 可见；CI 全绿
