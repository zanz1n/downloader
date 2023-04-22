package main

import (
	"encoding/json"
	"fmt"
	"log"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/cors"
	"github.com/gofiber/fiber/v2/middleware/logger"
	"github.com/gofiber/fiber/v2/middleware/recover"
	"github.com/zanz1n/downloader/node/services"
)

func main() {
	config := services.GetConfig()

	app := fiber.New(fiber.Config{
		Prefork:       false,
		ServerHeader:  "Fiber",
		CaseSensitive: true,
		StrictRouting: false,
		JSONEncoder:   json.Marshal,
		JSONDecoder:   json.Unmarshal,
	})

	app.Use(logger.New(logger.Config{
		Format:     "${pid} - ${time} [${ip}]:${port} ${method} ${path} ${status} ${latency}\n",
		TimeFormat: "2006/01/02 15:04:05",
		TimeZone:   "America/Sao_Paulo",
	}))

	app.Use(recover.New())
	app.Use(cors.New())

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
}
