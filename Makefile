build-node:
	go build -ldflags "-s -w" -o dist/node.bin github.com/zanz1n/downloader/node

build-proxy:
	go build -ldflags "-s -w" -o dist/proxy.bin github.com/zanz1n/downloader/proxy

run-node:
	go run github.com/zanz1n/downloader/node --config ./data/config.yml

run-proxy:
	go run github.com/zanz1n/downloader/proxy --env-file .env

test:
	go test ./apps/proxy/... --race
	go test ./apps/node/... --race
	go test ./sql/dba/... --race
	go test ./shared/... --race
