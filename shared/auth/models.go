package auth

import (
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/zanz1n/downloader/dba"
	"github.com/zanz1n/downloader/shared/errors"
)

type FileAccessPerm uint8

const (
	FileAccessPermRead  = 1
	FileAccessPermWrite = 2
)

type UserJwtPayload struct {
	UserID     string       `json:"sub"`
	Email      string       `json:"email"`
	ExpiryDate int64        `json:"exp"`
	IssuedAt   int64        `json:"iat"`
	Role       dba.UserRole `json:"role"`
}

func (p *UserJwtPayload) GetExpirationTime() (*jwt.NumericDate, error) {
	return &jwt.NumericDate{
		Time: time.Unix(p.ExpiryDate, 0),
	}, nil
}

func (p *UserJwtPayload) GetIssuedAt() (*jwt.NumericDate, error) {
	return &jwt.NumericDate{
		Time: time.Unix(p.IssuedAt, 0),
	}, nil
}

func (p *UserJwtPayload) GetNotBefore() (*jwt.NumericDate, error) {
	return nil, nil
}

func (p *UserJwtPayload) GetIssuer() (string, error) {
	return "", nil
}

func (p *UserJwtPayload) GetSubject() (string, error) {
	return p.UserID, nil
}

func (p *UserJwtPayload) GetAudience() (jwt.ClaimStrings, error) {
	return nil, nil
}

type FileAccessJwtPayload struct {
	FileID     string         `json:"sub"`
	ExpiryDate int64          `json:"exp"`
	IssuedAt   int64          `json:"iat"`
	Permission FileAccessPerm `json:"perm"`
}

func (p *FileAccessJwtPayload) Validate() error {
	if err := validate.Struct(p); err != nil {
		return errors.ErrInvalidJwtToken
	}

	if p.ExpiryDate < time.Now().Unix() {
		return errors.ErrExpiredJwtToken
	}

	return nil
}

func (p *FileAccessJwtPayload) GetExpirationTime() (*jwt.NumericDate, error) {
	return &jwt.NumericDate{
		Time: time.Unix(p.ExpiryDate, 0),
	}, nil
}

func (p *FileAccessJwtPayload) GetIssuedAt() (*jwt.NumericDate, error) {
	return &jwt.NumericDate{
		Time: time.Unix(p.IssuedAt, 0),
	}, nil
}

func (p *FileAccessJwtPayload) GetNotBefore() (*jwt.NumericDate, error) {
	return nil, nil
}

func (p *FileAccessJwtPayload) GetIssuer() (string, error) {
	return "", nil
}

func (p *FileAccessJwtPayload) GetSubject() (string, error) {
	return p.FileID, nil
}

func (p *FileAccessJwtPayload) GetAudience() (jwt.ClaimStrings, error) {
	return nil, nil
}
