-- name: GetNodeById :one
SELECT * FROM "nodes" WHERE "id" = $1;
