package tcp

import (
	"crypto/sha256"
	"encoding/hex"

	"github.com/google/uuid"
	"github.com/zanz1n/downloader/cmd/node/config"
	"github.com/zanz1n/downloader/internal/errors"
	"github.com/zanz1n/downloader/internal/logger"
	"github.com/zanz1n/downloader/internal/transport"
	"github.com/zanz1n/downloader/internal/utils"
)

func validateIden(iden *transport.IdenPayload) error {
	var err error
	if _, err = uuid.Parse(iden.ID); err != nil {
		return errors.ErrInvalidIdenPayload
	}
	key := config.GetConfig().GetKey()

	hash := sha256.New()

	if _, err = hash.Write(utils.S2B(iden.Random)); err != nil {
		logger.Error("Hashing failed: " + err.Error())
		return errors.ErrHashingFailed
	}
	if _, err = hash.Write(utils.S2B(key)); err != nil {
		logger.Error("Hashing failed: " + err.Error())
		return errors.ErrHashingFailed
	}

	buf := hash.Sum([]byte{})
	hexBuf := make([]byte, len(buf)*2)

	hex.Encode(hexBuf, buf)

	if string(hexBuf) != iden.Token {
		return errors.ErrInvalidSignature
	}

	return nil
}
