#!/bin/bash

# PostgreSQL数据库初始化脚本
# 用于创建WordMesh应用所需的表结构

set -e

echo "开始初始化PostgreSQL数据库..."

# 数据库连接信息
DB_HOST="localhost"
DB_PORT="5432"
DB_NAME="wordmesh_dev"
DB_USER="wordmesh"
DB_PASS="wordmesh123"

# 导出密码环境变量，避免psql提示输入密码
export PGPASSWORD=$DB_PASS

# 检查数据库连接
echo "检查数据库连接..."
if ! psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "SELECT 1;" > /dev/null 2>&1; then
    echo "错误：无法连接到PostgreSQL数据库"
    exit 1
fi

echo "数据库连接成功"

# 创建表结构
echo "创建表结构..."

# 创建用户表
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME << EOF
-- 用户表
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 单词表
CREATE TABLE IF NOT EXISTS words (
    id SERIAL PRIMARY KEY,
    word VARCHAR(100) UNIQUE NOT NULL,
    definition TEXT,
    part_of_speech VARCHAR(50),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 用户单词学习记录表
CREATE TABLE IF NOT EXISTS user_word_progress (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE,
    word_id INTEGER REFERENCES words(id) ON DELETE CASCADE,
    proficiency_level INTEGER DEFAULT 0, -- 熟练度等级 (0-5)
    last_reviewed TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    next_review TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, word_id)
);

-- 单词关系表 (用于存储同义词、反义词等关系)
CREATE TABLE IF NOT EXISTS word_relations (
    id SERIAL PRIMARY KEY,
    word_id INTEGER REFERENCES words(id) ON DELETE CASCADE,
    related_word_id INTEGER REFERENCES words(id) ON DELETE CASCADE,
    relation_type VARCHAR(50) NOT NULL, -- 'synonym', 'antonym', 'derivative', etc.
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(word_id, related_word_id, relation_type)
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_words_word ON words(word);
CREATE INDEX IF NOT EXISTS idx_user_word_progress_user_id ON user_word_progress(user_id);
CREATE INDEX IF NOT EXISTS idx_user_word_progress_next_review ON user_word_progress(next_review);
CREATE INDEX IF NOT EXISTS idx_word_relations_word_id ON word_relations(word_id);
CREATE INDEX IF NOT EXISTS idx_word_relations_related_word_id ON word_relations(related_word_id);

-- 创建更新时间触发器函数
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS \$\$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
\$\$ language 'plpgsql';

-- 为需要更新时间的表创建触发器
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_words_updated_at BEFORE UPDATE ON words FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_user_word_progress_updated_at BEFORE UPDATE ON user_word_progress FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
EOF

echo "PostgreSQL数据库初始化完成"