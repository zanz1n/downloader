package auth

import (
	"crypto/ed25519"
	"crypto/x509"
	"encoding/pem"
	"fmt"
	"os"

	"github.com/go-playground/validator/v10"
	"github.com/zanz1n/downloader/shared/logger"
)

var (
	authLogger = logger.NewLogger("auth")
	validate   = validator.New()
)

func MustGetEdDSAPemKeypair(
	privkeyPath, pubkeyPath string,
) (ed25519.PrivateKey, ed25519.PublicKey) {
	privkey, pubkey, err := GetEdDSAPemKeypair(privkeyPath, pubkeyPath)

	if err != nil {
		logger.Fatal("Key Pair: " + err.Error())
	}

	return privkey, pubkey
}

func GetEdDSAPemKeypair(
	privkeyPath, pubkeyPath string,
) (ed25519.PrivateKey, ed25519.PublicKey, error) {
	privkeyPem, err := os.ReadFile(privkeyPath)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to open private key at '%s'", privkeyPath)
	}

	pubkeyPem, err := os.ReadFile(pubkeyPath)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to open public key at '%s'", privkeyPath)
	}

	privBlock, _ := pem.Decode(privkeyPem)
	if privBlock == nil {
		return nil, nil, fmt.Errorf("private key is not a valid pem key")
	}

	pubBlock, _ := pem.Decode(pubkeyPem)
	if pubBlock == nil {
		return nil, nil, fmt.Errorf("private key is not a valid pem key")
	}

	privkey, err := x509.ParsePKCS8PrivateKey(privBlock.Bytes)
	if err != nil {
		return nil, nil, fmt.Errorf("private key is not a valid PKCS8 key")
	}

	pubkey, err := x509.ParsePKIXPublicKey(privBlock.Bytes)
	if err != nil {
		return nil, nil, fmt.Errorf("public key is not a valid PKIX key")
	}

	var (
		privEdKey ed25519.PrivateKey
		pubEdKey  ed25519.PublicKey
		ok        bool
	)
	if privEdKey, ok = privkey.(ed25519.PrivateKey); !ok {
		return nil, nil, fmt.Errorf("private key is not a EdDSA key")
	}
	if pubEdKey, ok = pubkey.(ed25519.PublicKey); !ok {
		return nil, nil, fmt.Errorf("public key is not a EdDSA key")
	}

	return privEdKey, pubEdKey, nil
}
