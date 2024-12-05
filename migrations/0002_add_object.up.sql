-- Add up migration script here

CREATE TABLE object (
    id blob PRIMARY KEY,
    user_id blob NOT NULL,
    created_at integer NOT NULL,
    updated_at integer NOT NULL,
    name text NOT NULL,
    mime_type text NOT NULL,
    size integer NOT NULL,
    checksum_256 blob NOT NULL
) STRICT;

CREATE INDEX object_user_id_idx ON object(user_id);
