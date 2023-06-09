package main

import (
	"encoding/json"
	"fmt"
	"log"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/cors"
	"github.com/gofiber/fiber/v2/middleware/recover"
	"github.com/zanz1n/downloader/node/routes"
	"github.com/zanz1n/downloader/node/services"
	"github.com/zanz1n/downloader/shared/logger"
)

var endCh = make(chan os.Signal)

func init() {
	logger.Init()
}

func main() {
	config := services.GetConfig()

	app := fiber.New(fiber.Config{
		Prefork:               false,
		ServerHeader:          "Fiber",
		CaseSensitive:         true,
		StrictRouting:         false,
		JSONEncoder:           json.Marshal,
		JSONDecoder:           json.Unmarshal,
		StreamRequestBody:     true,
		DisableStartupMessage: true,
	})

	app.Hooks().OnListen(func() error {
		logger.Info("Listenning on port %v with %v handlers", config.Port, app.HandlersCount())
		return nil
	})

	app.Hooks().OnShutdown(func() error {
		logger.Info("Shutting down ...")
		return nil
	})

	app.Use(logger.NewFiberMiddleware())

	app.Use(recover.New())
	app.Use(cors.New())

	routes.NewRouter(app)

	signal.Notify(endCh, syscall.SIGTERM, syscall.SIGINT, os.Interrupt)

	go func() {
		if config.UseSSL {
			err := app.ListenTLS(fmt.Sprintf("0.0.0.0:%v", config.Port), config.SSLCertPath, config.SSLKeyPath)
			if err != nil {
				log.Fatalln(err)
			}
		} else {
			err := app.Listen(fmt.Sprintf("0.0.0.0:%v", config.Port))
			if err != nil {
				log.Fatalln(err)
			}
		}
	}()

	<-endCh
	app.ShutdownWithTimeout(3 * time.Second)
}
