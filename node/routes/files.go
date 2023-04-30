package routes

import (
	"fmt"
	"io"
	"os"
	"strings"

	"github.com/gofiber/fiber/v2"
	"github.com/zanz1n/downloader/node/services"
)

func GetFile(jp *services.JwtService) func(c *fiber.Ctx) error {
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

func PostFile(jp *services.JwtService) func(c *fiber.Ctx) error {
	config := services.GetConfig()

	return func(c *fiber.Ctx) error {
		/* Authorization */

		id := c.Params("id")

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

		if token == "" {
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
				"error": "autorization header is required",
			})
		}

		validatedJwt, err := jp.ValidateFileSig(token)

		if err != nil {
			return c.Status(fiber.StatusUnauthorized).JSON(fiber.Map{
				"error": "an invalid token was provided",
			})
		}

		if validatedJwt.FileId != id || !strings.Contains(string(validatedJwt.Permission), "W") {
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
				"error": "the requested token doesn't grant you write access to this file",
			})
		}

		/* File handling */

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
