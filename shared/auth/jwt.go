package auth

import (
	"context"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/zanz1n/downloader/dba"
	"github.com/zanz1n/downloader/shared/errors"
	"github.com/zanz1n/downloader/shared/utils"
	"golang.org/x/crypto/bcrypt"
)

type Options struct {
	UserTokenDuration time.Duration
	JwtKey            []byte
}

func NewAuthService(dba dba.Querier, options *Options) *AuthService {
	return &AuthService{
		Options:       options,
		dba:           dba,
		signingMethod: jwt.SigningMethodHS256,
	}
}

type AuthService struct {
	*Options
	userTokenDuration time.Duration
	dba               dba.Querier
	signingMethod     jwt.SigningMethod
}

func (as *AuthService) EncodeFileAccessToken(claims *FileAccessJwtPayload) (string, error) {
	token := jwt.NewWithClaims(as.signingMethod, claims)

	s, err := token.SignedString(as.JwtKey)

	if err != nil {
		authLogger.Warn("Failed to generate file access jwt token: " + err.Error())
		return "", errors.ErrTokenGenerationFailed
	}

	return s, nil
}

func (as *AuthService) EncodeUserToken(claims *UserJwtPayload) (string, error) {
	token := jwt.NewWithClaims(as.signingMethod, claims)

	s, err := token.SignedString(as.JwtKey)

	if err != nil {
		authLogger.Warn("Failed to generate user jwt token: " + err.Error())
		return "", errors.ErrTokenGenerationFailed
	}

	return s, nil
}

func (as *AuthService) DecodeFileAccessToken(payload string) (*FileAccessJwtPayload, error) {
	claims := FileAccessJwtPayload{}

	token, err := jwt.ParseWithClaims(payload, &claims, as.accessTokenKeyFunc)

	if err != nil || !token.Valid {
		return nil, errors.ErrInvalidJwtToken
	}

	if err = claims.Validate(); err != nil {
		return nil, err
	}

	return &claims, nil
}

func (as *AuthService) CreateFileAccessToken(
	fileId string,
	permission FileAccessPerm,
	duration time.Duration,
) (string, error) {
	now := time.Now()
	expiry := now.Add(duration)

	claims := FileAccessJwtPayload{
		FileID:     fileId,
		ExpiryDate: expiry.Unix(),
		IssuedAt:   now.Unix(),
		Permission: permission,
	}

	return as.EncodeFileAccessToken(&claims)
}

func (as *AuthService) AuthUser(email, passwd string) (string, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 7*time.Second)
	defer cancel()

	info, err := as.dba.GetJwtInfoByEmail(ctx, email)

	if err != nil {
		return "", errors.ErrUserAuthFailed
	}

	err = bcrypt.CompareHashAndPassword(
		utils.S2B(info.Password),
		utils.S2B(passwd),
	)

	if err != nil {
		return "", errors.ErrUserAuthFailed
	}

	now := time.Now()
	expiry := now.Add(as.userTokenDuration)

	claims := UserJwtPayload{
		UserID:     info.ID,
		Email:      info.Email,
		ExpiryDate: expiry.Unix(),
		IssuedAt:   now.Unix(),
		Role:       info.Role,
	}

	return as.EncodeUserToken(&claims)
}

func (as *AuthService) accessTokenKeyFunc(token *jwt.Token) (interface{}, error) {
	if _, ok := token.Method.(*jwt.SigningMethodHMAC); !ok {
		return nil, errors.ErrInvalidJwtToken
	}

	return as.JwtKey, nil
}
