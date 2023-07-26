package main

import (
	"flag"
	"os"
	"os/signal"
	"syscall"

	"github.com/zanz1n/downloader/apps/node/config"
	"github.com/zanz1n/downloader/apps/node/server"
	"github.com/zanz1n/downloader/shared/logger"
)

var (
	configFile = flag.String("config", "/etc/downloader/config.yml", "The configuration file")
	debug      = flag.Bool("debug", false, "If set, the app print debug logs")
)

func init() {
	flag.Parse()

	if *debug {
		logger.SetLevel("debug")
	}

	config.MustFromYamlFile(*configFile)
}

func main() {
	endCh := make(chan os.Signal, 1)

	srv := server.NewServer()

	signal.Notify(endCh, syscall.SIGINT, syscall.SIGTERM, os.Interrupt)

	go srv.MustListenAndServe(":8080")

	<-endCh

	srv.Shutdown()
}
