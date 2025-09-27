-- Word module initial schema
CREATE TABLE IF NOT EXISTS words (
    id BIGSERIAL PRIMARY KEY,
    text VARCHAR(128) NOT NULL,
    canonical_key VARCHAR(160) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT words_canonical_key_unique UNIQUE (canonical_key)
);

CREATE TABLE IF NOT EXISTS user_words (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    word_id BIGINT NOT NULL REFERENCES words(id) ON DELETE CASCADE,
    tags TEXT[] NOT NULL DEFAULT '{}',
    note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT user_words_user_id_word_id_unique UNIQUE (user_id, word_id)
);

CREATE TABLE IF NOT EXISTS user_senses (
    id BIGSERIAL PRIMARY KEY,
    user_word_id BIGINT NOT NULL REFERENCES user_words(id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order INT NOT NULL DEFAULT 0,
    note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT user_senses_user_word_id_text_unique UNIQUE (user_word_id, text)
);

CREATE UNIQUE INDEX IF NOT EXISTS user_senses_primary_unique
    ON user_senses (user_word_id)
    WHERE is_primary;

CREATE INDEX IF NOT EXISTS idx_user_words_user_id ON user_words (user_id);
CREATE INDEX IF NOT EXISTS idx_user_words_word_id ON user_words (word_id);
CREATE INDEX IF NOT EXISTS idx_user_words_tags ON user_words USING GIN (tags);
CREATE INDEX IF NOT EXISTS idx_user_senses_user_word_id ON user_senses (user_word_id);
CREATE INDEX IF NOT EXISTS idx_user_senses_sort_order ON user_senses (sort_order);
