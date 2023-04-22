package routes

import (
	"github.com/gofiber/fiber/v2"
)

func NewRouter(app *fiber.App) {
	app.Get("/stats", GetStats())
}
