-- Add up migration script here

CREATE TABLE user (
    id blob PRIMARY KEY,
    created_at integer NOT NULL,
    updated_at integer NOT NULL,
    permission integer NOT NULL,
    username text NOT NULL,
    password text NOT NULL
) STRICT;

CREATE UNIQUE INDEX user_username_idx ON user(username);
