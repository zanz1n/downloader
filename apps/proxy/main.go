package main

import (
	"context"
	"fmt"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/zanz1n/downloader/dba"
	"github.com/zanz1n/downloader/proxy/config"
	"github.com/zanz1n/downloader/proxy/repository/auth"
	"github.com/zanz1n/downloader/proxy/repository/user"
	"github.com/zanz1n/downloader/proxy/server"
	"github.com/zanz1n/downloader/shared/logger"
	"github.com/zanz1n/downloader/shared/utils"
)

func init() {
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
}
