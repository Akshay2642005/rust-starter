-- Add migration script here

CREATE TABLE todos (
    id      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title   TEXT NOT NULL,
    done    BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
