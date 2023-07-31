package main

import (
	"flag"
	"fmt"
	"os"
	"os/signal"
	"syscall"

	"github.com/zanz1n/downloader/node/config"
	"github.com/zanz1n/downloader/node/server"
	"github.com/zanz1n/downloader/node/tcp"
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
	signal.Notify(endCh, syscall.SIGINT, syscall.SIGTERM, os.Interrupt)

	cfg := config.GetConfig()

	srv := server.NewServer()
	tsrv := tcp.NewServer()

	if cfg.App.SSL.Enabled {
		go srv.MustListenAndServeTLS(
			fmt.Sprintf(":%v", cfg.App.Port),
			cfg.App.SSL.CertificateFile,
			cfg.App.SSL.KeyFile,
		)

		if cfg.App.TCP.Enabled {
			go tsrv.MustListenAndServeTLS(
				fmt.Sprintf(":%v", cfg.App.TCP.Port),
				cfg.App.SSL.CertificateFile,
				cfg.App.SSL.KeyFile,
			)
		}
	} else {
		go srv.MustListenAndServe(fmt.Sprintf(":%v", cfg.App.Port))

		if cfg.App.TCP.Enabled {
			go tsrv.MustListenAndServe(fmt.Sprintf(":%v", cfg.App.TCP.Port))
		}
	}

	<-endCh

	config.DumpToFile()
	srv.Shutdown()
}
