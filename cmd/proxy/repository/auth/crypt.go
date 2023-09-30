package auth

import (
	"github.com/zanz1n/downloader/internal/errors"
	"github.com/zanz1n/downloader/internal/utils"
	"golang.org/x/crypto/bcrypt"
)

const BcryptSaltLength = 12

func HashPassword(passwd string) (string, error) {
	hash, err := bcrypt.GenerateFromPassword(utils.S2B(passwd), BcryptSaltLength)

	if err != nil {
		authLogger.Error("Failed to hash user password: " + err.Error())
		return "", errors.ErrPasswordHashingFailed
	}

	return utils.B2S(hash), nil
}
