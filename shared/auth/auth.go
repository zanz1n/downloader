package auth

import (
	"context"
	"crypto/ed25519"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/zanz1n/downloader/dba"
	"github.com/zanz1n/downloader/shared/errors"
	"github.com/zanz1n/downloader/shared/utils"
	"golang.org/x/crypto/bcrypt"
)

type Options struct {
	UserTokenDuration time.Duration
	JwtHmacKey        []byte
	JwtEdDSAPrivKey   ed25519.PrivateKey
	JwtEdDSAPubKey    ed25519.PublicKey
}

func NewAuthService(dba dba.Querier, options *Options) *AuthService {
	return &AuthService{
		Options:     options,
		dba:         dba,
		simSigning:  jwt.SigningMethodHS256,
		asimSigning: jwt.SigningMethodEdDSA,
	}
}

type AuthService struct {
	*Options
	dba               dba.Querier
	simSigning        jwt.SigningMethod
	asimSigning       jwt.SigningMethod
}

func (as *AuthService) EncodeFileAccessToken(claims *FileAccessJwtPayload) (string, error) {
	token := jwt.NewWithClaims(as.simSigning, claims)

	s, err := token.SignedString(as.JwtHmacKey)

	if err != nil {
		authLogger.Warn("Failed to generate file access jwt token: " + err.Error())
		return "", errors.ErrTokenGenerationFailed
	}

	return s, nil
}

func (as *AuthService) EncodeUserToken(claims *UserJwtPayload) (string, error) {
	token := jwt.NewWithClaims(as.asimSigning, claims)

	s, err := token.SignedString(as.JwtEdDSAPrivKey)

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

func (as *AuthService) DecodeUserToken(payload string) (*UserJwtPayload, error) {
	claims := UserJwtPayload{}

	token, err := jwt.ParseWithClaims(payload, &claims, as.userTokenKeyFunc)

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
	expiry := now.Add(as.UserTokenDuration)

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

	return as.JwtHmacKey, nil
}

func (as *AuthService) userTokenKeyFunc(token *jwt.Token) (interface{}, error) {
	if _, ok := token.Method.(*jwt.SigningMethodEd25519); !ok {
		return nil, errors.ErrInvalidJwtToken
	}

	return as.JwtEdDSAPubKey, nil
}
