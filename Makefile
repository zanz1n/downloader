build-node:
	go build -ldflags "-s -w" -o dist/node.bin ./node/main.go

run-node:
	go run ./node/main.go
