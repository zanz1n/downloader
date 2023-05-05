package routes

import (
	"fmt"
	"os"

	"github.com/gofiber/fiber/v2"
	"github.com/zanz1n/downloader/node/services"
)

func DeleteFile(as *services.AuthService) func(c *fiber.Ctx) error {
	config := services.GetConfig()

	return func(c *fiber.Ctx) error {
		id := c.Params("id")

		jwtAuth := c.Query("token")

		authErr := as.AuthFileWrite(id, jwtAuth)

		if authErr != nil {
			return c.Status(authErr.Status()).JSON(fiber.Map{
				"error": authErr.Error(),
			})
		}

		err := os.Remove(fmt.Sprintf("%s/%s", config.DataPath, id))

		if err != nil {
			return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
				"error": fmt.Sprintf("file %s was not found", id),
			})
		}

		return c.JSON(fiber.Map{
			"message": fmt.Sprintf("file %s was deleted", id),
		})
	}
}
