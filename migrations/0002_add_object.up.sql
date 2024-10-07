-- Add up migration script here

CREATE TABLE object (
    id blob PRIMARY KEY,
    created_at integer NOT NULL,
    updated_at integer NOT NULL,
    name text NOT NULL,
    mime_type text NOT NULL,
    size integer NOT NULL,
    checksum_256 blob NOT NULL
) STRICT;
