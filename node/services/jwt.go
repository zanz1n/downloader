package services

import (
	"errors"
	"fmt"
	"time"

	"github.com/golang-jwt/jwt/v4"
)

type JwtService struct {
	token string
}

type FileSigJwtPayload struct {
	FileId string `json:"file_id"`
}

func NewJwtService() *JwtService {
	return &JwtService{
		token: GetConfig().JwtKey,
	}
}

func (j *JwtService) ValidateFileSig(p string) (*FileSigJwtPayload, error) {
	var fileId string

	token, err := jwt.Parse(p, func(t *jwt.Token) (interface{}, error) {
		var ok bool
		
		if _, ok = t.Method.(*jwt.SigningMethodHMAC); !ok {
			return nil, fmt.Errorf("unexpected jwt signing method: %s", t.Header["alg"].(string))
		}

		if fileId, ok = t.Claims.(jwt.MapClaims)["file_id"].(string); !ok {
			return nil, fmt.Errorf("invalid token payload")
		}

		if t.Claims.(jwt.MapClaims)["exp"].(float64) < float64(time.Now().Unix()) {
			return nil, errors.New("token is expired")
		}

		return []byte(j.token), nil
	})

	if err != nil || !token.Valid {
		return nil, err
	}

	return &FileSigJwtPayload{
		FileId: fileId,
	}, nil
}
