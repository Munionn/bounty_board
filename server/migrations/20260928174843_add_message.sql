CREATE TYPE file_status AS ENUM (
    'pending',
    'uploaded',
    'failed',
    'deleted'
);

CREATE TABLE files (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_wallet        TEXT NOT NULL REFERENCES users (wallet_address) ON UPDATE CASCADE,
    bucket              TEXT NOT NULL,
    object_key          TEXT NOT NULL UNIQUE,
    object_version_id   TEXT,
    original_filename   TEXT NOT NULL,
    content_type        TEXT,
    size_bytes          BIGINT CHECK (size_bytes IS NULL OR size_bytes >= 0),
    checksum_sha256     TEXT,
    status              file_status NOT NULL DEFAULT 'pending',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    uploaded_at         TIMESTAMPTZ,
    deleted_at          TIMESTAMPTZ
);

CREATE TABLE chats (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bounty_id   UUID NOT NULL UNIQUE REFERENCES bounties (id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE messages (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    chat_id         UUID NOT NULL REFERENCES chats (id) ON DELETE CASCADE,
    sender_wallet   TEXT NOT NULL REFERENCES users (wallet_address) ON UPDATE CASCADE,
    body            TEXT NOT NULL DEFAULT '',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE message_files (
    message_id  UUID NOT NULL REFERENCES messages (id) ON DELETE CASCADE,
    file_id     UUID NOT NULL REFERENCES files (id) ON DELETE CASCADE,
    position    SMALLINT NOT NULL DEFAULT 0,
    caption     TEXT,
    PRIMARY KEY (message_id, file_id)
);

CREATE INDEX idx_files_owner_created_at ON files (owner_wallet, created_at DESC);
CREATE INDEX idx_files_status ON files (status);
CREATE INDEX idx_chats_bounty_id ON chats (bounty_id);
CREATE INDEX idx_messages_chat_created_at ON messages (chat_id, created_at DESC);
CREATE INDEX idx_messages_sender_wallet ON messages (sender_wallet);
