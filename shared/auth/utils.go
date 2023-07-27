package auth

import (
	"github.com/go-playground/validator/v10"
	"github.com/zanz1n/downloader/shared/logger"
)

var (
	authLogger = logger.NewLogger("auth")
	validate = validator.New()
)
