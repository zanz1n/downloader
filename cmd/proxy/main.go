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
	"github.com/joho/godotenv"
	"github.com/zanz1n/downloader/cmd/proxy/config"
	"github.com/zanz1n/downloader/cmd/proxy/repository/auth"
	"github.com/zanz1n/downloader/cmd/proxy/repository/user"
	"github.com/zanz1n/downloader/cmd/proxy/server"
	"github.com/zanz1n/downloader/internal/dba"
	"github.com/zanz1n/downloader/internal/logger"
	"github.com/zanz1n/downloader/internal/utils"
)

var envFile = flag.String("env-file", "", "The environment variables file")

func init() {
	flag.Parse()

	if *envFile != "" {
		godotenv.Load(*envFile)
	}

	config.MustFromEnv()
}

func main() {
	endCh := make(chan os.Signal, 1)
	signal.Notify(endCh, syscall.SIGINT, syscall.SIGTERM, os.Interrupt)

	cfg := config.GetConfig()

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	conn, err := pgx.Connect(ctx, cfg.PostgresURI)

	if err != nil {
		logger.Fatal("Failed to connect to database: " + err.Error())
	}

	db := dba.New(conn)

	jwtPrivkey, jwtPubkey := auth.MustGetEdDSAPemKeypair(
		cfg.JWTPrivkeyPath,
		cfg.JWTPubkeyPath,
	)

	authService := auth.NewAuthService(db, &auth.Options{
		UserTokenDuration: time.Hour,
		JwtHmacKey:        utils.S2B(cfg.JWTHmacKey),
		JwtEdDSAPrivKey:   jwtPrivkey,
		JwtEdDSAPubKey:    jwtPubkey,
	})

	userService := user.NewUserService(db)

	srv := server.NewServer(db, authService, userService)

	if cfg.EnableTLS {
		go srv.MustListenAndServeTLS(
			fmt.Sprintf(":%v", cfg.Port),
			cfg.TLSCertPath,
			cfg.TLSKeyPath,
		)
	} else {
		go srv.MustListenAndServe(fmt.Sprintf(":%v", cfg.Port))
	}

	<-endCh

	srv.Shutdown()
	conn.Close(context.Background())
}
