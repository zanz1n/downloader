build-node:
	go build -ldflags "-s -w" -o dist/node.bin ./apps/node/main.go

build-proxy:
	go build -ldflags "-s -w" -o dist/proxy.bin ./apps/proxy/main.go

run-node:
	go run ./apps/node/main.go --config ./data/config.yml

run-proxy:
	go run ./apps/proxy/main.go

test:
	go test ./apps/proxy/... --race
	go test ./apps/node/... --race
	go test ./sql/dba/... --race
	go test ./shared/... --race
