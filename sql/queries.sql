-- name: GetNodeById :one
SELECT * FROM "nodes" WHERE "id" = $1;

-- name: GetUserByEmail :one
SELECT * FROM "users" WHERE "email" = $1;

-- name: GetUserByID :one
SELECT * FROM "users" WHERE "id" = $1;

-- name: GetJwtInfoByEmail :one
SELECT "id", "email", "password", "role", "deleted" FROM "users" WHERE "email" = $1;

-- name: GetFileAuthInfo :one
SELECT "id", "name", "checksum", "userId", "nodeId", "contentType" FROM "files" WHERE id = $1;

-- name: CreateUser :one
INSERT INTO "users" ("id", "firstName", "lastName", "email", "password", "role") VALUES ($1, $2, $3, $4, $5, $6) RETURNING *;
