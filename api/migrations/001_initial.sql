-- Notes module — initial schema
-- Run automatically by the API on startup.

CREATE TABLE IF NOT EXISTS notes (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id   TEXT        NOT NULL,        -- Authentik uid
    title      TEXT        NOT NULL DEFAULT '',
    content    TEXT        NOT NULL DEFAULT '',
    color      TEXT        NOT NULL DEFAULT '#ffffff',
    pinned     BOOLEAN     NOT NULL DEFAULT false,
    archived   BOOLEAN     NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS todos (
    id       UUID    PRIMARY KEY DEFAULT gen_random_uuid(),
    note_id  UUID    NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    text     TEXT    NOT NULL DEFAULT '',
    checked  BOOLEAN NOT NULL DEFAULT false,
    position INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS note_shares (
    id                UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    note_id           UUID        NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    shared_with_id    TEXT        NOT NULL,  -- Authentik uid of the recipient
    shared_with_email TEXT        NOT NULL,
    permission        TEXT        NOT NULL DEFAULT 'view',  -- 'view' | 'edit'
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(note_id, shared_with_id)
);

CREATE INDEX IF NOT EXISTS idx_notes_owner   ON notes(owner_id);
CREATE INDEX IF NOT EXISTS idx_shares_target ON note_shares(shared_with_id);
