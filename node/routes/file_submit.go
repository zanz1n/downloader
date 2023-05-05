package routes

import (
	"fmt"
	"io"
	"os"

	"github.com/gofiber/fiber/v2"
	"github.com/zanz1n/downloader/node/services"
)

func PostFile(as *services.AuthService) func(c *fiber.Ctx) error {
	config := services.GetConfig()

	return func(c *fiber.Ctx) error {
		id := c.Params("id")

		authHeader := c.Get("Authorization")


		authErr := as.AuthFileWrite(id, authHeader)

		if authErr != nil {
			return c.Status(authErr.Status()).JSON(fiber.Map{
				"error": authErr.Error(),
			})
		}

		file, err := c.FormFile("file")

		if err != nil {
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
				"error": "invalid form body",
			})
		}

		fileExists, err := os.Open(fmt.Sprintf("%s/%s", config.DataPath, file.Filename))

		if err == nil {
			fileExists.Close()
			return c.Status(fiber.StatusConflict).JSON(fiber.Map{
				"error": "the submited file already exists",
			})
		}

		dst, err := os.Create(fmt.Sprintf("%s/%s", config.DataPath, file.Filename))

		if err != nil {
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
				"error": "malformed file id, probably invalid UTF-8 characters",
			})
		}

		dstName := dst.Name()

		src, err := file.Open()

		if err != nil {
			os.Remove(dstName)
			dst.Close()
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
				"error": "invalid form body",
			})
		}

		defer src.Close()
		defer dst.Close()

		writtenBytes, err := io.Copy(dst, src)

		if err != nil {
			return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
				"error": err.Error(),
			})
		}

		return c.JSON(fiber.Map{
			"message": fmt.Sprintf("sucess, written %v bytes", writtenBytes),
		})
	}
}
