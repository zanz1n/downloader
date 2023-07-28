package main

import (
	"context"
	"flag"
	"fmt"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/zanz1n/downloader/apps/node/config"
	"github.com/zanz1n/downloader/apps/node/server"
	"github.com/zanz1n/downloader/dba"
	"github.com/zanz1n/downloader/shared/auth"
	"github.com/zanz1n/downloader/shared/logger"
	"github.com/zanz1n/downloader/shared/utils"
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

	connCtx, connCancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer connCancel()

	conn, err := pgx.Connect(connCtx, cfg.PostgresURI)

	if err != nil {
		logger.Fatal("Failed to connect to database: " + err.Error())
	}

	db := dba.New(conn)

	jwtPrivkey, jwtPubkey := auth.MustGetEdDSAPemKeypair(
		cfg.Jwt.PrivKey,
		cfg.Jwt.PubKey,
	)

	authService := auth.NewAuthService(db, &auth.Options{
		UserTokenDuration: time.Hour,
		JwtHmacKey:        utils.S2B(cfg.Jwt.Hkey),
		JwtEdDSAPrivKey:   jwtPrivkey,
		JwtEdDSAPubKey:    jwtPubkey,
	})

	srv := server.NewServer(db, authService)

	if cfg.App.SSL.Enabled {
		go srv.MustListenAndServeTLS(
			fmt.Sprintf(":%v", cfg.App.Port),
			cfg.App.SSL.CertificateFile,
			cfg.App.SSL.KeyFile,
		)
	} else {
		go srv.MustListenAndServe(fmt.Sprintf(":%v", cfg.App.Port))
	}

	<-endCh

	config.DumpToFile()
	srv.Shutdown()
}
