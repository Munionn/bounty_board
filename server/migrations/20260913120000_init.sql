CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TYPE bounty_status AS ENUM ('open', 'claimed', 'closed');

CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address  TEXT NOT NULL UNIQUE,
    display_name    TEXT,
    bio             TEXT,
    reputation      BIGINT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE bounties (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    on_chain_id         BIGINT NOT NULL,
    poster_wallet       TEXT NOT NULL REFERENCES users (wallet_address) ON UPDATE CASCADE,
    claimer_wallet      TEXT REFERENCES users (wallet_address) ON UPDATE CASCADE,
    pda_address         TEXT NOT NULL UNIQUE,
    title               TEXT NOT NULL,
    description         TEXT NOT NULL DEFAULT '',
    amount_lamports     BIGINT NOT NULL CHECK (amount_lamports >= 0),
    status              bounty_status NOT NULL DEFAULT 'open',
    submission_uri      TEXT,
    tx_signature        TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (poster_wallet, on_chain_id)
);

CREATE TABLE submissions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bounty_id           UUID NOT NULL REFERENCES bounties (id) ON DELETE CASCADE,
    submitter_wallet    TEXT NOT NULL REFERENCES users (wallet_address) ON UPDATE CASCADE,
    submission_uri      TEXT NOT NULL,
    note                TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (bounty_id, submitter_wallet)
);

CREATE INDEX idx_bounties_status ON bounties (status);
CREATE INDEX idx_bounties_poster ON bounties (poster_wallet);
CREATE INDEX idx_bounties_claimer ON bounties (claimer_wallet);
CREATE INDEX idx_bounties_created_at ON bounties (created_at DESC);
CREATE INDEX idx_submissions_bounty ON submissions (bounty_id);
CREATE INDEX idx_submissions_submitter ON submissions (submitter_wallet);

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER users_set_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER bounties_set_updated_at
    BEFORE UPDATE ON bounties
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();
