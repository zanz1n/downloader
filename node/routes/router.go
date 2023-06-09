package routes

import (
	"github.com/gofiber/fiber/v2"
	"github.com/zanz1n/downloader/node/middlewares"
	"github.com/zanz1n/downloader/node/services"
	"github.com/zanz1n/downloader/shared/auth"
)

func NewRouter(app *fiber.App) {
	app.Use("/stats", middlewares.NewProtectedMiddleware())
	app.Get("/stats", GetStats())

	jwtProvider := auth.NewJwtService(services.GetConfig().JwtKey)

	authService := services.NewAuthService(jwtProvider)

	app.Get("/file/:id", GetFile(jwtProvider))
	app.Post("/file/:id", PostFile(authService))
	app.Delete("/file/:id", DeleteFile(authService))
}
