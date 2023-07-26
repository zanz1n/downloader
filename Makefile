build-node:
	go build -ldflags "-s -w" -o dist/node.bin ./apps/node/main.go

run-node:
	go run ./apps/node/main.go --config ./data/config.yml
