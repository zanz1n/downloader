package auth

import (
	"crypto/ed25519"
	"os"

	"github.com/zanz1n/downloader/shared/logger"
)

var authLogger = logger.NewLogger("auth")

func MustReadJwtKeyPair(privKeyFile, pubKeyFile string) (ed25519.PrivateKey, ed25519.PublicKey) {
	jwtPriv, err := os.ReadFile(privKeyFile)
	if err != nil {
		authLogger.Fatal(
			"Failed to open jwt private key file at '%s': %s",
			privKeyFile,
			err.Error(),
		)
	}

	jwtPub, err := os.ReadFile(pubKeyFile)
	if err != nil {
		authLogger.Fatal(
			"Failed to open jwt public key file at '%s': %s",
			pubKeyFile,
			err.Error(),
		)
	}

	return jwtPriv, jwtPub
}
