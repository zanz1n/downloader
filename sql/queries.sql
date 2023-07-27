-- name: GetNodeById :one
SELECT * FROM "nodes" WHERE "id" = $1;

-- name: GetUserByEmail :one
SELECT * FROM "users" WHERE "email" = $1;

-- name: GetUserByID :one
SELECT * FROM "users" WHERE "id" = $1;

-- name: GetJwtInfoByEmail :one
SELECT "id", "email", "password", "role" FROM "users" WHERE "email" = $1;

-- name: GetFileUserAndNodeById :one
SELECT "userId", "nodeId" FROM "files" WHERE id = $1;
