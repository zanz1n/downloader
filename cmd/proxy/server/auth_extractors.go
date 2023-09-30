package server

import (
	"strings"

	"github.com/valyala/fasthttp"
	"github.com/zanz1n/downloader/cmd/proxy/repository/auth"
	"github.com/zanz1n/downloader/internal/errors"
	"github.com/zanz1n/downloader/internal/utils"
)

type AuthHeader struct {
	SigningType string
	Token       string
}

type FileAuthorization struct {
	IsSignedToken bool
	IsUserToken   bool
	UserID        string
	TokenPerm     auth.FileAccessPerm
}

func (s *Server) ExtractFileAuthorization(
	ctx *fasthttp.RequestCtx,
	fileId string,
) (*FileAuthorization, error) {
	authHeader, err := s.ExtractAuthorizationHeader(ctx)
	if err != nil {
		return nil, err
	}

	if authHeader == nil {
		authQuery := ctx.URI().QueryArgs().PeekBytes([]byte("access_token"))

		if authQuery == nil {
			return nil, errors.ErrRouteRequiresAuth
		}

		claims, err := s.as.DecodeFileAccessToken(utils.B2S(authQuery))

		if err != nil {
			return nil, err
		}

		if claims.FileID != fileId {
			return nil, errors.ErrFileAccessDenied
		}

		return &FileAuthorization{
			IsSignedToken: true,
			IsUserToken:   false,
			UserID:        "",
			TokenPerm:     claims.Permission,
		}, nil
	} else {
		if authHeader.SigningType == "Bearer" {
			claims, err := s.as.DecodeUserToken(authHeader.Token)

			if err != nil {
				return nil, err
			}

			return &FileAuthorization{
				IsSignedToken: false,
				IsUserToken:   true,
				UserID:        claims.UserID,
				TokenPerm:     0,
			}, nil
		} else {
			return nil, errors.ErrInvalidAuthHeader
		}
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
