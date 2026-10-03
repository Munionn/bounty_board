-- migrate: no transaction
ALTER TYPE file_status ADD VALUE IF NOT EXISTS 'processing';

ALTER TABLE files
    ADD COLUMN IF NOT EXISTS bounty_id UUID REFERENCES bounties (id) ON DELETE CASCADE,
    ADD COLUMN IF NOT EXISTS chat_id UUID REFERENCES chats (id) ON DELETE CASCADE,
    ADD COLUMN IF NOT EXISTS preview_object_key TEXT,
    ADD COLUMN IF NOT EXISTS extracted_text TEXT;

CREATE INDEX IF NOT EXISTS idx_files_bounty_id ON files (bounty_id);
CREATE INDEX IF NOT EXISTS idx_files_chat_id ON files (chat_id);
