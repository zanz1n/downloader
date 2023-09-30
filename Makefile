build-node:
	go build -ldflags "-s -w" -o bin/node github.com/zanz1n/downloader/cmd/node

build-proxy:
	go build -ldflags "-s -w" -o bin/proxy github.com/zanz1n/downloader/cmd/proxy

run-node:
	go run github.com/zanz1n/downloader/cmd/node --config ./data/config.yml

run-proxy:
	go run github.com/zanz1n/downloader/cmd/proxy --env-file .env

test:
	go test ./... -v --race
