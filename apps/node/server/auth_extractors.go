package server

import (
	"context"
	"crypto/sha256"
	"encoding/base64"
	"strings"
	"time"

	"github.com/valyala/fasthttp"
	"github.com/zanz1n/downloader/apps/node/config"
	"github.com/zanz1n/downloader/dba"
	"github.com/zanz1n/downloader/shared/auth"
	"github.com/zanz1n/downloader/shared/errors"
	"github.com/zanz1n/downloader/shared/logger"
	"github.com/zanz1n/downloader/shared/utils"
)

type AuthHeader struct {
	SigningType string
	Token       string
}

func (s *Server) ExtractFileAuthorization(
	ctx *fasthttp.RequestCtx,
	fileId string,
) (auth.FileAccessPerm, *dba.GetFileAuthInfoRow, error) {
	authHeader, err := s.ExtractAuthorizationHeader(ctx)
	if err != nil {
		return 0, nil, err
	}

	if authHeader == nil {
		authQuery := ctx.URI().QueryArgs().PeekBytes([]byte("access_token"))

		if authQuery == nil {
			return 0, nil, errors.ErrRouteRequiresAuth
		}

		claims, err := s.as.DecodeFileAccessToken(utils.B2S(authQuery))

		if err != nil {
			return 0, nil, err
		}

		if claims.FileID != fileId {
			return 0, nil, errors.ErrFileAccessDenied
		}

		info, err := s.extractFileInfoById(fileId)

		if err != nil {
			return 0, nil, err
		}
		cfg := config.GetConfig()

		switch {
		case info.NodeId != cfg.InstanceID:
			return 0, nil, errors.ErrFileNotLocatedInNode
		case claims.Permission == auth.FileAccessPermRead:
			return auth.FileAccessPermRead, info, nil
		case claims.Permission == auth.FileAccessPermWrite:
			return auth.FileAccessPermWrite, info, nil
		default:
			return 0, nil, errors.ErrFileAccessDenied
		}
	} else {
		if authHeader.SigningType == "Bearer" {
			claims, err := s.as.DecodeUserToken(authHeader.Token)

			if err != nil {
				return 0, nil, err
			}

			info, err := s.extractFileInfoById(fileId)

			if err != nil {
				return 0, nil, err
			}
			cfg := config.GetConfig()

			switch {
			case info.NodeId != cfg.InstanceID:
				return 0, nil, errors.ErrFileNotLocatedInNode
			case info.UserId == claims.UserID || claims.Role == dba.UserRoleADMIN:
				return auth.FileAccessPermWrite, info, nil
			default:
				return 0, nil, errors.ErrFileAccessDenied
			}
		} else {
			return 0, nil, errors.ErrInvalidAuthHeader
		}
	}
}

func (s *Server) ExtractFileWriteAuthorization(
	ctx *fasthttp.RequestCtx,
	fileId string,
) (*dba.GetFileAuthInfoRow, error) {
	perm, info, err := s.ExtractFileAuthorization(ctx, fileId)

	if err != nil {
		return nil, err
	}

	if perm == auth.FileAccessPermWrite {
		return info, nil
	} else {
		return nil, errors.ErrFileAccessDenied
	}
}

func (s *Server) ExtractFileReadAuthorization(
	ctx *fasthttp.RequestCtx,
	fileId string,
) (*dba.GetFileAuthInfoRow, error) {
	perm, info, err := s.ExtractFileAuthorization(ctx, fileId)

	if err != nil {
		return nil, err
	}

	if perm == auth.FileAccessPermRead || perm == auth.FileAccessPermWrite {
		return info, nil
	} else {
		return nil, errors.ErrFileAccessDenied
	}
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
	if _, err := hash.Write(utils.S2B(config.GetConfig().Key)); err != nil {
		logger.Error("Hashing failed: " + err.Error())
		return errors.ErrHashingFailed
	}

	buf := hash.Sum([]byte{})
	base64Buf := []byte{}

	base64.StdEncoding.Encode(base64Buf, buf)

	if string(base64Buf) != authHeader.Token {
		return errors.ErrInvalidSignature
	}
	return nil
}

func (s *Server) extractFileInfoById(id string) (*dba.GetFileAuthInfoRow, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	info, err := s.db.GetFileAuthInfo(ctx, id)

	if err != nil {
		return nil, errors.ErrFileNotFound
	}

	return info, nil
}
