package middlewares

import (
	"crypto/sha256"
	"encoding/base64"
	"log"

	"github.com/gofiber/fiber/v2"
	"github.com/zanz1n/downloader/node/services"
)

func NewProtectedMiddleware() func(c *fiber.Ctx) error {
	config := services.GetConfig()

	return func(c *fiber.Ctx) error {
		hash := sha256.New()

		authHeader := c.Get("Signature")

		if len(authHeader) < 20 {
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
				"error": "invalid signature format",
			})
		}

		hash.Write(c.Body())

		sum := hash.Sum([]byte(config.ManagerKey))

		result := base64.StdEncoding.EncodeToString(sum)

		log.Println(string(sum))

		if authHeader != result {
			return c.Status(fiber.StatusUnauthorized).JSON(fiber.Map{
				"error": "invalid signature",
			})
		}

		return c.Next()
	}
}
