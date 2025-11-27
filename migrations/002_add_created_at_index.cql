-- add secondary index on created_at for metrics queries
-- removes the need for ALLOW FILTERING which is slow in ScyllaDB

CREATE INDEX IF NOT EXISTS users_created_at_idx ON equicloud.users (created_at);
