package server

import (
	"crypto/sha256"
	"encoding/hex"
	"strings"

	"github.com/valyala/fasthttp"
	"github.com/zanz1n/downloader/node/config"
	"github.com/zanz1n/downloader/shared/errors"
	"github.com/zanz1n/downloader/shared/logger"
	"github.com/zanz1n/downloader/shared/utils"
)

type AuthHeader struct {
	SigningType string
	Token       string
}

func (s *Server) ExtractAuthorizationHeader(c *fasthttp.RequestCtx) (*AuthHeader, error) {
	authHeader := c.Request.Header.Peek("Authorization")

	if authHeader == nil {
		return nil, nil
	}

	authHeaderS := strings.Split(utils.B2S(authHeader), " ")
	if len(authHeaderS) != 2 {
		return nil, errors.ErrInvalidAuthHeader
	}

	return &AuthHeader{
		SigningType: authHeaderS[0],
		Token:       authHeaderS[1],
	}, nil
}

func (s *Server) ExtractSignatureAuthorization(c *fasthttp.RequestCtx, p []byte) error {
	authHeader, err := s.ExtractAuthorizationHeader(c)
	if err != nil {
		return err
	} else if authHeader == nil {
		return errors.ErrRouteRequiresAuth
	}

	hash := sha256.New()
	if _, err := hash.Write(p); err != nil {
		logger.Error("Hashing failed: " + err.Error())
		return errors.ErrHashingFailed
	}
	if _, err := hash.Write(utils.S2B(config.GetConfig().GetKey())); err != nil {
		logger.Error("Hashing failed: " + err.Error())
		return errors.ErrHashingFailed
	}

	buf := hash.Sum([]byte{})
	hexBuf := make([]byte, len(buf)*2)

	hex.Encode(hexBuf, buf)

	if string(hexBuf) != authHeader.Token {
		return errors.ErrInvalidSignature
	}
	return nil
}
