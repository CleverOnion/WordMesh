# WordMesh

<div align="center">

A personal knowledge network builder for connecting words and concepts to construct your semantic world.

[English](#english) | [简体中文](#简体中文)

[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org)
[![Next.js](https://img.shields.io/badge/Next.js-16-black.svg)](https://nextjs.org)
[![TypeScript](https://img.shields.io/badge/TypeScript-5-blue.svg)](https://www.typescriptlang.org)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

</div>

---

## English

### Overview

WordMesh is a bilingual (Chinese/English) web application that helps users build personal knowledge networks through word connections and semantic associations. It uses a dual-database architecture with PostgreSQL for structured data and Neo4j for graph relationships.

### Features

- **User Management**: Registration, login with JWT authentication
- **Knowledge Cards**: Add personal definitions and translations to words
- **Semantic Network**: Connect concepts through word-to-word and sense-to-word associations
- **Visual Exploration**: Graph-based visualization of knowledge connections (in progress)

### Tech Stack

| Layer | Technology |
|--------|-------------|
| **Backend** | Rust + Actix Web + SQLx + neo4rs |
| **Frontend** | Next.js 16 + React 19 + TypeScript + Tailwind CSS + shadcn/ui |
| **Database** | PostgreSQL 16 (relational) + Neo4j 5.15 (graph) |
| **Auth** | JWT (HS256/RS256) + bcrypt password hashing |

### Project Structure

```
WordMesh/
├── WordMesh-backend/       # Rust backend service
│   ├── src/
│   │   ├── controller/     # HTTP interface layer
│   │   ├── service/        # Business logic layer
│   │   ├── repository/     # Data access layer
│   │   ├── domain/         # Domain models
│   │   └── middleware/     # Auth, request tracing
│   └── migrations/        # Database migrations
├── wordmesh-frontend/      # Next.js web application
│   ├── src/
│   │   ├── app/           # App Router pages
│   │   ├── modules/        # Feature modules (word, sense, association)
│   │   ├── components/     # UI components
│   │   ├── shared/        # API client, utilities
│   │   └── hooks/         # Custom React hooks
│   └── public/            # Static assets
├── deployment/             # Docker Compose configurations
└── docs/                   # API documentation, design docs
```

### Quick Start

#### Prerequisites

- Docker and Docker Compose
- Rust 1.70+ (for backend development)
- Node.js 20+ (for frontend development)

#### 1. Start Database Services

```bash
cd deployment
docker compose up -d
```

This starts PostgreSQL on port 5432 and Neo4j on ports 7474 (HTTP) and 7687 (Bolt).

#### 2. Start Backend Service

```bash
cd WordMesh-backend
cargo run
```

Backend API will be available at `http://localhost:8080`

#### 3. Start Frontend Application

```bash
cd wordmesh-frontend
npm install
npm run dev
```

Frontend application will be available at `http://localhost:3000`

#### Access Points

| Service | URL |
|---------|-----|
| Frontend | http://localhost:3000 |
| Backend API | http://localhost:8080 |
| Neo4j Browser | http://localhost:7474 |
| Health Check | http://localhost:8080/api/v1/health |

### API Usage

#### Register User

```bash
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"demo_user","password":"demo123456"}'
```

#### Login

```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"demo_user","password":"demo123456"}'
```

All endpoints return a unified response format:

```json
{
  "code": 2000,
  "message": "OK",
  "data": { ... },
  "traceId": "uuid",
  "timestamp": 1234567890
}
```

### Documentation

- **[CLAUDE.md](./CLAUDE.md)** - Development guide (architecture, commands, coding standards)
- **[API Documentation](./docs/api.md)** - Complete API reference
- **[Backend README](./docs/backend/README.md)** - Backend architecture details

### Development Status

| Module | Status |
|--------|--------|
| Authentication | ✅ Complete |
| Word Management | 🔄 In Progress (~60%) |
| Associations | 🔄 In Progress (~40%) |
| Network Visualization | 🚧 Planned |

---

## 简体中文

### 概述

WordMesh 是一个双语（中文/英文）Web 应用，帮助用户通过单词连接和语义关联构建个人知识网络。项目采用双数据库架构，PostgreSQL 存储结构化数据，Neo4j 存储图关系。

### 功能特性

- **用户管理**：注册、登录，JWT 认证
- **知识卡片**：为单词添加个人定义和翻译
- **语义网络**：通过词-词和义-词关联连接概念
- **可视化探索**：基于图形的知识连接可视化（开发中）

### 技术栈

| 层级 | 技术 |
|------|------|
| **后端** | Rust + Actix Web + SQLx + neo4rs |
| **前端** | Next.js 16 + React 19 + TypeScript + Tailwind CSS + shadcn/ui |
| **数据库** | PostgreSQL 16（关系型）+ Neo4j 5.15（图数据库）|
| **认证** | JWT (HS256/RS256) + bcrypt 密码哈希 |

### 快速开始

#### 前置要求

- Docker 和 Docker Compose
- Rust 1.70+（后端开发）
- Node.js 20+（前端开发）

#### 1. 启动数据库服务

```bash
cd deployment
docker compose up -d
```

这将启动 PostgreSQL（端口 5432）和 Neo4j（端口 7474 HTTP、7687 Bolt）。

#### 2. 启动后端服务

```bash
cd WordMesh-backend
cargo run
```

后端 API 将在 `http://localhost:8080` 运行。

#### 3. 启动前端应用

```bash
cd wordmesh-frontend
npm install
npm run dev
```

前端应用将运行在 `http://localhost:3000`。

#### 访问地址

| 服务 | 地址 |
|------|------|
| 前端应用 | http://localhost:3000 |
| 后端 API | http://localhost:8080 |
| Neo4j 浏览器 | http://localhost:7474 |
| 健康检查 | http://localhost:8080/api/v1/health |

### 文档

- **[CLAUDE.md](./CLAUDE.md)** - 开发指南（架构说明、命令、代码规范）
- **[API 文档](./docs/api.md)** - 完整 API 参考
- **[后端 README](./docs/backend/README.md)** - 后端架构详情

### 开发进度

| 模块 | 状态 |
|------|------|
| 身份认证 | ✅ 已完成 |
| 单词管理 | 🔄 开发中 (~60%) |
| 关联管理 | 🔄 开发中 (~40%) |
| 网络可视化 | 🚧 计划中 |

---

## License

MIT License - see [LICENSE](LICENSE) for details.
