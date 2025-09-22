CREATE TABLE IF NOT EXISTS equicloud.users (
    id TEXT PRIMARY KEY,
    settings BLOB,
    created_at BIGINT,
    updated_at BIGINT
);