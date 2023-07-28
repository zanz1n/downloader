package server

import (
	"context"
	"strings"
	"time"

	"github.com/valyala/fasthttp"
	"github.com/zanz1n/downloader/apps/node/config"
	"github.com/zanz1n/downloader/dba"
	"github.com/zanz1n/downloader/shared/auth"
	"github.com/zanz1n/downloader/shared/errors"
	"github.com/zanz1n/downloader/shared/utils"
)

func (s *Server) extractFileInfoById(id string) (*dba.GetFileAuthInfoRow, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	info, err := s.db.GetFileAuthInfo(ctx, id)

	if err != nil {
		return nil, errors.ErrFileNotFound
	}

	return info, nil
}

func (s *Server) ExtractFileAuthorization(
	ctx *fasthttp.RequestCtx,
	fileId string,
) (auth.FileAccessPerm, *dba.GetFileAuthInfoRow, error) {
	authHeader := ctx.Request.Header.PeekBytes([]byte("authorization"))

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
		if len(authHeader) < 11 {
			return 0, nil, errors.ErrInvalidAuthHeader
		}

		authHeaderS := strings.Split(utils.B2S(authHeader), " ")
		if len(authHeaderS) != 2 {
			return 0, nil, errors.ErrInvalidAuthHeader
		}

		method, token := authHeaderS[0], authHeaderS[1]

		if method == "Bearer" {
			claims, err := s.as.DecodeUserToken(token)

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
