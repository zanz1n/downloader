package services

import (
	"strings"

	"github.com/gofiber/fiber/v2"
	"github.com/zanz1n/downloader/shared/auth"
)

type AuthService struct {
	jwt *auth.JwtService
}

func NewAuthService(j *auth.JwtService) *AuthService {
	return &AuthService{
		jwt: j,
	}
}

func (as *AuthService) AuthFileWrite(id string, authToken string) AuthRejectErr {
	if authToken == "" {
		return NewAuthRejectErr(
			"the authorization header was not provided",
			fiber.StatusBadRequest,
		)
	}

	if !strings.HasPrefix(authToken, "Bearer ") {
		return NewAuthRejectErr(
			"malformed authorization header",
			fiber.StatusBadRequest,
		)
	}

	token := strings.Replace(authToken, "Bearer ", "", 1)

	if token == "" {
		return NewAuthRejectErr(
			"autorization header is required",
			fiber.StatusBadRequest,
		)
	}

	validatedJwt, err := as.jwt.ValidateFileSig(token)

	if err != nil {
		return NewAuthRejectErr(
			"an invalid token was provided",
			fiber.StatusUnauthorized,
		)
	}

	if validatedJwt.FileId != id || !strings.Contains(string(validatedJwt.Permission), "W") {
		return NewAuthRejectErr(
			"the requested token doesn't grant you write access to this file",
			fiber.StatusBadRequest,
		)
	}

	return nil
}
