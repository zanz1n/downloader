-- name: GetJwtInfoByEmail :one
SELECT "id", "email", "password", "role", "deleted" FROM "users" WHERE "email" = $1;

-- name: GetFileAndNodeInfo :one
SELECT "files"."id",
    "files"."name",
    "files"."contentType",
    "files"."checksum",
    "files"."userId",
    "nodes"."id" AS "nodeId",
    "nodes"."address" AS "nodeAddress",
    "nodes"."port" AS "nodePort",
    "nodes"."ssl" AS "nodeSSL",
    "nodes"."tcpPort" AS "nodeTCPPort"
FROM "files"
INNER JOIN "nodes" ON "files"."nodeId" = "nodes"."id"
WHERE "files"."id" = $1;

-- name: GetFileAuthInfo :one
SELECT "id", "name", "checksum", "userId", "nodeId", "contentType" FROM "files" WHERE "id" = $1;

-- name: CreateUser :one
INSERT INTO "users" ("id", "firstName", "lastName", "email", "password", "role") VALUES ($1, $2, $3, $4, $5, $6) RETURNING *;

-- name: CreateFile :one
INSERT INTO "files" ("id", "name", "contentType", "size", "checksum", "nodeId", "userId") VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *;
