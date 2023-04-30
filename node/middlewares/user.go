package middlewares

import (
	"strings"

	"github.com/gofiber/fiber/v2"
	"github.com/zanz1n/downloader/node/services"
)

func NewUserOnlyMiddleware(jp *services.JwtService) func(c *fiber.Ctx) error {
	return func(c *fiber.Ctx) error {
		authHeader := c.Get("Authorization")

		if authHeader == "" {
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
				"error": "the authorization header was not provided",
			})
		}

		if !strings.HasPrefix(authHeader, "Bearer ") {
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
				"error": "malformed authorization header",
			})
		}

		token := strings.Replace(authHeader, "Bearer ", "", 1)

		user, err := jp.ValidateUser(token)

		if err != nil {
			return c.Status(fiber.StatusUnauthorized).JSON(fiber.Map{
				"error": err.Error(),
			})
		}

		c.Locals("user", user)

		return c.Next()
	}
}
