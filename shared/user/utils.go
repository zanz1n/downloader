package user

import (
	"regexp"

	"github.com/go-playground/validator/v10"
)

var (
	mailRegex = regexp.MustCompile(`^[\w-\.]+@([\w-]+\.)+[\w-]{2,4}$`)
	validate  = validator.New()
)

func IsWeakPassword(passwd string) bool {
	return false
}
