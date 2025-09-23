#!/bin/bash
set -e

# 初始化PostgreSQL数据库表结构
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    -- 创建用户表
    CREATE TABLE IF NOT EXISTS users (
        id BIGSERIAL PRIMARY KEY,
        username VARCHAR(255) UNIQUE NOT NULL,
        password VARCHAR(255) NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    -- 创建单词表
    CREATE TABLE IF NOT EXISTS words (
        id BIGSERIAL PRIMARY KEY,
        text VARCHAR(255) UNIQUE NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    -- 创建义项表
    CREATE TABLE IF NOT EXISTS senses (
        id BIGSERIAL PRIMARY KEY,
        user_id BIGINT REFERENCES users(id),
        word_id BIGINT REFERENCES words(id),
        definition TEXT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    -- 创建笔记表
    CREATE TABLE IF NOT EXISTS notes (
        id BIGSERIAL PRIMARY KEY,
        user_id BIGINT REFERENCES users(id),
        node_type VARCHAR(50) NOT NULL,
        node_id BIGINT NOT NULL,
        content TEXT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    -- 创建索引以提高查询性能
    CREATE INDEX IF NOT EXISTS idx_senses_user_id ON senses(user_id);
    CREATE INDEX IF NOT EXISTS idx_senses_word_id ON senses(word_id);
    CREATE INDEX IF NOT EXISTS idx_notes_user_id ON notes(user_id);
    CREATE INDEX IF NOT EXISTS idx_notes_node_type_node_id ON notes(node_type, node_id);
EOSQL