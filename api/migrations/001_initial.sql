-- Notes module — initial schema

CREATE TABLE IF NOT EXISTS notes (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id   TEXT        NOT NULL,        -- Authentik user.uid (JWT sub)
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

-- Shares are keyed by email because the API receives email at share time.
-- shared_with_id is populated lazily when the recipient first accesses the note.
CREATE TABLE IF NOT EXISTS note_shares (
    id                UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    note_id           UUID        NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    shared_with_email TEXT        NOT NULL,
    shared_with_id    TEXT,                  -- Authentik uid; set on first access
    permission        TEXT        NOT NULL DEFAULT 'view',  -- 'view' | 'edit'
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(note_id, shared_with_email)
);

CREATE INDEX IF NOT EXISTS idx_notes_owner        ON notes(owner_id);
CREATE INDEX IF NOT EXISTS idx_shares_email       ON note_shares(shared_with_email);
CREATE INDEX IF NOT EXISTS idx_shares_uid         ON note_shares(shared_with_id);
