package server

import (
	"strings"

	"github.com/valyala/fasthttp"
	"github.com/zanz1n/downloader/dba"
	"github.com/zanz1n/downloader/shared/auth"
	"github.com/zanz1n/downloader/shared/errors"
	"github.com/zanz1n/downloader/shared/utils"
)

func (s *Server) ExtractFileAuthorization(
	ctx *fasthttp.RequestCtx,
	fileId, userId string,
) (auth.FileAccessPerm, error) {
	authHeader := ctx.Request.Header.PeekBytes([]byte("authorization"))

	if authHeader == nil {
		authQuery := ctx.URI().QueryArgs().PeekBytes([]byte("access_token"))

		if authQuery == nil {
			return 0, errors.ErrRouteRequiresAuth
		}

		claims, err := s.as.DecodeFileAccessToken(utils.B2S(authQuery))

		if err != nil {
			return 0, err
		}

		if claims.FileID != fileId {
			return 0, errors.ErrFileAccessDenied
		}

		if claims.Permission == auth.FileAccessPermRead {
			return auth.FileAccessPermRead, nil
		} else if claims.Permission == auth.FileAccessPermWrite {
			return auth.FileAccessPermWrite, nil
		} else {
			return 0, errors.ErrFileAccessDenied
		}
	} else {
		if len(authHeader) < 11 {
			return 0, errors.ErrInvalidAuthHeader
		}

		authHeaderS := strings.Split(utils.B2S(authHeader), " ")
		if len(authHeaderS) != 2 {
			return 0, errors.ErrInvalidAuthHeader
		}

		method, token := authHeaderS[0], authHeaderS[1]

		if method == "Bearer" {
			claims, err := s.as.DecodeUserToken(token)

			if err != nil {
				return 0, err
			}

			if claims.UserID == userId || claims.Role == dba.UserRoleADMIN {
				return auth.FileAccessPermWrite, nil
			} else {
				return 0, errors.ErrFileAccessDenied
			}
		} else {
			return 0, errors.ErrInvalidAuthHeader
		}
	}
}

func (s *Server) ExtractFileWriteAuthorization(
	ctx *fasthttp.RequestCtx,
	fileId, userId string,
) error {
	perm, err := s.ExtractFileAuthorization(ctx, fileId, userId)

	if err != nil {
		return err
	}

	if perm == auth.FileAccessPermWrite {
		return nil
	} else {
		return errors.ErrFileAccessDenied
	}
}

func (s *Server) ExtractFileReadAuthorization(
	ctx *fasthttp.RequestCtx,
	fileId, userId string,
) error {
	perm, err := s.ExtractFileAuthorization(ctx, fileId, userId)

	if err != nil {
		return err
	}

	if perm == auth.FileAccessPermRead || perm == auth.FileAccessPermWrite {
		return nil
	} else {
		return errors.ErrFileAccessDenied
	}
}
