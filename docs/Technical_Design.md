# WordMesh 后端技术设计文档

## 1. 技术选型

考虑到性能、安全性和项目核心需求的匹配度，我们选择以下技术栈来构建 WordMesh 的后端服务。

- **编程语言: Rust**
  - **理由**: Rust 提供内存安全保证，避免了常见的并发问题，其高性能特性非常适合构建响应迅速的 API 服务。项目已包含 Rust 的配置文件，表明团队具备相应的技术能力。

- **Web 框架: Actix Web**
  - **理由**: Actix Web 是一个功能强大、成熟且性能极高的 Web 框架。它基于 Actor 模型构建，提供了出色的并发性能和一套丰富的中间件生态系统，非常适合构建高性能的 API 服务。

- **数据库组合**: 
  - **关系型数据库: PostgreSQL**
    - **理由**: PostgreSQL 用于存储结构化的用户账户数据。它功能强大、可靠且开源，非常适合处理用户认证和管理等事务性操作。
  - **图数据库: Neo4j**
    - **理由**: 项目的核心是构建一个网状的知识结构。Neo4j 作为领先的图数据库，能够原生存储和高效查询节点与关系，完美匹配本项目的需求，查询单词关联（如查找一个词的所有同义词）的性能和表达能力远超传统关系型数据库。

- **数据库驱动**: 
  - **PostgreSQL驱动: SQLx**
    - **理由**: SQLx 是一个异步的、编译时检查的 Rust SQL 工具包。它能在编译阶段就验证 SQL 查询的正确性，极大地提高了代码的健壮性。
  - **Neo4j驱动: `neo4rs`**
    - **理由**: `neo4rs` 是一个社区维护的、功能完善的异步 Neo4j Bolt 协议驱动，适合在 Axum 中使用。

## 2. 架构设计

我们将采用分层架构，结合领域驱动设计（DDD）的思想来组织代码，以实现高度模块化、可维护和可扩展的系统。

### 2.1. 项目目录结构

```
src/
├─ main.rs              # 程序入口，负责初始化、配置加载和启动 Actix Web 服务
├─ config/              # 配置管理模块 (dotenv, settings)
├─ controller/          # API 接口层 (Actix Handlers)
├─ application/         # 应用服务层 (Use Cases)
├─ service/             # 领域服务层 (Domain Services)
├─ repository/          # 仓储层 (Data Access)
├─ domain/              # 领域模型层 (Entities, Value Objects)
├─ event/               # 事件 & 消息处理 (未来扩展)
├─ middleware/          # 中间件 (Authentication, Logging)
├─ dto/                 # 数据传输对象 (Request/Response Structs)
└─ util/                # 通用工具模块
```

### 2.2. 各层职责

- **`main.rs`**: 应用的启动入口。负责读取配置、初始化数据库连接池、设置日志、构建 Actix 应用实例并挂载路由和中间件。

- **`config`**: 负责加载和管理应用配置。使用 `dotenv` 从 `.env` 文件加载环境变量，并用一个强类型的结构体来管理所有配置项。

- **`controller` (API 接口层)**: 接收 HTTP 请求，验证输入（通常是 `dto`），调用 `application` 层的用例来执行业务操作，并处理返回结果，将其序列化为 HTTP 响应。这一层是 Actix Web 的 `Handler` 所在的位置。

- **`application` (应用服务层)**: 编排一个或多个领域服务 (`service`) 和仓储 (`repository`) 来完成一个完整的业务用例 (Use Case)。例如，“用户注册”用例会调用用户服务来创建用户实体，然后通过仓储将用户保存到数据库。

- **`service` (领域服务层)**: 包含不属于任何单个实体的核心业务逻辑。这些服务封装了复杂的业务规则或与外部系统的交互。

- **`repository` (仓储层)**: 数据访问的抽象。定义了与数据库交互的接口（Traits），具体的实现（例如，使用 `sqlx` 和 `neo4rs` 的实现）将与这些接口解耦。它负责将领域对象 (`domain`) 持久化和检索。

- **`domain` (领域模型层)**: 项目的核心。包含了业务的实体（Entities）、值对象（Value Objects）和领域事件。这一层不依赖任何其他层，是纯粹的业务逻辑表示。

- **`event`**: 用于处理领域事件或与其他系统进行消息传递。这是一个为未来功能（如异步通知、CQRS）预留的模块。

- **`middleware`**: Actix Web 的中间件。用于处理跨多个请求的通用逻辑，例如用户认证、请求日志记录、CORS 设置等。

- **`dto` (数据传输对象)**: 定义了 API 的请求体和响应体的结构。它们是 `controller` 层与外部世界交互的数据契约，有助于将 API 的数据结构与内部的领域模型 (`domain`) 分离开。

- **`util`**: 存放项目范围内的通用函数、常量或宏，例如自定义的错误处理、密码哈希工具等。

## 3. 数据库设计

我们采用混合持久化策略。PostgreSQL 作为数据的主要来源 (Source of Truth)，负责存储结构化的核心数据（用户、单词、义项、笔记）。Neo4j 则专注于存储和查询这些数据之间的复杂关系网络。

### 3.1. PostgreSQL 数据库设计

**1. `users` - 用户表** (无变化)

| 字段名      | 类型          | 约束                | 描述           |
| ----------- | ------------- | ------------------- | -------------- |
| `id`        | `BIGSERIAL`   | `PRIMARY KEY`       | 用户唯一标识   |
| `username`  | `VARCHAR(255)`| `UNIQUE NOT NULL`   | 用户名         |
| `password`      | `VARCHAR(255)` | `NOT NULL`          | 用户密码 (临时方案, **生产环境必须哈希**) |
| `created_at`| `TIMESTAMPTZ` | `NOT NULL DEFAULT NOW()` | 创建时间       |

**2. `words` - 单词表**

| 字段名      | 类型          | 约束                | 描述           |
| ----------- | ------------- | ------------------- | -------------- |
| `id`        | `BIGSERIAL`   | `PRIMARY KEY`       | 单词唯一标识         |
| `text`      | `VARCHAR(255)`| `UNIQUE NOT NULL`   | 单词的文本 (如 "run") |
| `created_at`| `TIMESTAMPTZ` | `NOT NULL DEFAULT NOW()` | 创建时间       |

**3. `senses` - 义项表**

| 字段名        | 类型          | 约束                | 描述                 |
| ------------- | ------------- | ------------------- | -------------------- |
| `id`          | `BIGSERIAL`   | `PRIMARY KEY`       | 义项唯一标识         |
| `user_id`     | `BIGINT`      | `REFERENCES users(id)` | 所属用户 ID          |
| `word_id`     | `BIGINT`      | `REFERENCES words(id)` | 所属单词 ID          |
| `definition`  | `TEXT`        | `NOT NULL`          | 义项的中文定义 (如 "跑") |
| `created_at`  | `TIMESTAMPTZ` | `NOT NULL DEFAULT NOW()` | 创建时间             |

**4. `notes` - 笔记表**

| 字段名        | 类型          | 约束           | 描述                               |
| ------------- | ------------- | -------------- | ---------------------------------- |
| `id`          | `BIGSERIAL`   | `PRIMARY KEY`  | 笔记唯一标识                       |
| `user_id`     | `BIGINT`      | `REFERENCES users(id)` | 所属用户 ID                        |
| `node_type`   | `VARCHAR(50)` | `NOT NULL`     | 节点类型 ("word" 或 "sense")       |
| `node_id`     | `BIGINT`      | `NOT NULL`     | 节点 ID (指向 `words.id` 或 `senses.id`) |
| `content`     | `TEXT`        | `NOT NULL`     | 笔记内容                           |
| `created_at`  | `TIMESTAMPTZ` | `NOT NULL DEFAULT NOW()` | 创建时间                           |

### 3.2. Neo4j 图数据库设计

Neo4j 中的节点是 PostgreSQL 中数据的“镜像”，但只包含足以构建关系图的 ID。

**节点 (Nodes):**

1.  `:Word`
    - **描述**: 代表一个单词，其元数据存储在 PostgreSQL 的 `words` 表中。
    - **属性**: 
        - `word_id`: `BIGINT` (UNIQUE) - 对应 PostgreSQL `words.id` 的外键。

2.  `:Sense`
    - **描述**: 代表一个义项，其元数据存储在 PostgreSQL 的 `senses` 表中。
    - **属性**: 
        - `sense_id`: `BIGINT` (UNIQUE) - 对应 PostgreSQL `senses.id` 的外键。
        - `user_id`: `BIGINT` - 对应 `users.id`，冗余字段用于按用户查询优化。

**关系 (Relationships):**

在我们的模型中，**所有关系本质上都是用户私有的**，这反映了用户个人的知识网络。我们不再区分“全局”和“私有”关系，取而代之的是一个统一的所有权模型：

- **隐式私有关系**: 如果一个关系的起点或终点是 `:Sense` 节点，那么这个关系的所有权由 `:Sense` 节点的 `user_id` 隐式决定。因为 `:Sense` 节点本身是用户私有的，任何与之相连的关系自然也属于该用户。
- **显式私有关系**: 如果一个关系连接的是两个公共的 `:Word` 节点，那么这个关系**必须**包含一个 `user_id` 属性来明确其所有者。

**关系类型:**

*   `:DEFINES` (Sense -> Word): 链接一个义项到它所定义的单词。这是一个隐式私有关系。
    *   示例: `MATCH (s:Sense {sense_id: 1, user_id: 101}), (w:Word {word_id: 55}) CREATE (s)-[:DEFINES]->(w)`
*   `:RELATED_TO` (Sense -> Sense): 连接两个用户认为相关的义项（可以是同义、反义或其他联想）。这是一个隐式私有关系。
    *   示例: `MATCH (s1:Sense {sense_id: 2, user_id: 101}), (s2:Sense {sense_id: 3, user_id: 101}) CREATE (s1)-[:RELATED_TO]->(s2)`
*   `:LINKED_WORD` (Word -> Word): **带属性的关系**，表示某个用户在两个单词之间建立了个人链接。这是一个显式私有关系。
    *   **属性**:
        *   `user_id` (BIGINT, Indexed): 创建此链接的 `users.id`。
    *   **示例**: `MATCH (w1:Word {word_id: 10}), (w2:Word {word_id: 25}) CREATE (w1)-[:LINKED_WORD {user_id: 101}]->(w2)`

**核心设计思想:**

该模型确保了数据的完全隔离。用户的知识网络（他们创建的义项和关系）完全属于他们自己。公共的 `:Word` 节点作为知识的“锚点”，而用户通过私有的 `:Sense` 节点和带 `user_id` 的关系在这些锚点之上构建自己的语义空间。这种设计既能利用共享词汇表的效率，又能保证用户数据的绝对私密性和个性化。

**查询示例:**

*   查找用户 101 创建的所有关系:
    ```cypher
    // 查找连接到该用户私有 Sense 节点的关系 (隐式)
    MATCH (:Sense {user_id: 101})-[r]-()
    RETURN r
    UNION
    // 查找该用户在 Word 节点之间建立的关系 (显式)
    MATCH (:Word)-[r {user_id: 101}]->(:Word)
    RETURN r
    ```