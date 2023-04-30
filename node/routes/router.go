package routes

import (
	"github.com/gofiber/fiber/v2"
	"github.com/zanz1n/downloader/node/middlewares"
	"github.com/zanz1n/downloader/node/services"
)

func NewRouter(app *fiber.App) {
	app.Use("/stats", middlewares.NewProtectedMiddleware())
	app.Get("/stats", GetStats())

	jwtProvider := services.NewJwtService()

	app.Get("/file/:id", GetFile(jwtProvider))
	app.Post("/file/:id", PostFile(jwtProvider))
}
