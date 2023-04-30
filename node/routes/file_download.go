package routes

import (
	"fmt"
	"os"

	"github.com/gofiber/fiber/v2"
	"github.com/zanz1n/downloader/node/services"
	"github.com/zanz1n/downloader/shared/auth"
)

func GetFile(jp *auth.JwtService) func(c *fiber.Ctx) error {
	config := services.GetConfig()

	return func(c *fiber.Ctx) error {
		id := c.Params("id")

		jwtAuth := c.Query("token")

		if jwtAuth == "" {
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
				"error": "token query param is required",
			})
		}

		validatedJwt, err := jp.ValidateFileSig(jwtAuth)

		if err != nil {
			return c.Status(fiber.StatusUnauthorized).JSON(fiber.Map{
				"error": "an invalid token was provided",
			})
		}

		if validatedJwt.FileId != id {
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
				"error": "the requested file does not match the jwt token info",
			})
		}

		file, err := os.Open(fmt.Sprintf("%s/%s", config.DataPath, id))

		if err != nil {
			return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
				"error": "the requested file was not found in the node",
			})
		}

		loc := file.Name()

		file.Close()

		return c.Download(loc)
	}
}
