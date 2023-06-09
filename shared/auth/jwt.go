package auth

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
	FileId     string         `json:"file_id"`
	Permission userPermission `json:"permission"`
}

type userPermission string

const UserPermissionRW userPermission = "RW"
const UserPermissionRead userPermission = "R"
const UserPermissionWrite userPermission = "W"

type UserJwtPayload struct {
	ID         string         `json:"id"`
	Username   string         `json:"username"`
	Permission userPermission `json:"permission"`
}

func NewJwtService(token string) *JwtService {
	return &JwtService{
		token: token,
	}
}

func (j *JwtService) GenerateFileSig(p *FileSigJwtPayload) (string, error) {
	token := jwt.NewWithClaims(jwt.SigningMethodHS512, jwt.MapClaims{
		"file_id":    p.FileId,
		"permission": p.Permission,
	})

	tokenString, err := token.SignedString([]byte(j.token))

	return tokenString, err
}

func (j *JwtService) ValidateFileSig(p string) (*FileSigJwtPayload, error) {
	var (
		fileId     string
		permission userPermission
	)

	token, err := jwt.Parse(p, func(t *jwt.Token) (interface{}, error) {
		var ok bool

		if _, ok = t.Method.(*jwt.SigningMethodHMAC); !ok {
			return nil, fmt.Errorf("unexpected jwt signing method: %s", t.Header["alg"].(string))
		}

		if fileId, ok = t.Claims.(jwt.MapClaims)["file_id"].(string); !ok {
			return nil, fmt.Errorf("invalid token payload")
		}

		if permission, ok = t.Claims.(jwt.MapClaims)["file_permission"].(userPermission); !ok {
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
		FileId:     fileId,
		Permission: permission,
	}, nil
}

func (j *JwtService) GenerateUserToken(p *UserJwtPayload) (string, error) {
	token := jwt.NewWithClaims(jwt.SigningMethodHS512, jwt.MapClaims{
		"id":         p.ID,
		"username":   p.Username,
		"permission": p.Permission,
	})

	tokenString, err := token.SignedString([]byte(j.token))

	return tokenString, err
}

func (j *JwtService) ValidateUser(p string) (*UserJwtPayload, error) {
	var (
		id         string
		username   string
		permission userPermission
	)

	token, err := jwt.Parse(p, func(t *jwt.Token) (interface{}, error) {
		var ok bool

		if _, ok = t.Method.(*jwt.SigningMethodHMAC); !ok {
			return nil, fmt.Errorf("unexpected jwt signing method: %s", t.Header["alg"].(string))
		}

		if username, ok = t.Claims.(jwt.MapClaims)["username"].(string); !ok {
			return nil, fmt.Errorf("invalid token payload")
		}

		if id, ok = t.Claims.(jwt.MapClaims)["id"].(string); !ok {
			return nil, fmt.Errorf("invalid token payload")
		}

		if permission, ok = t.Claims.(jwt.MapClaims)["id"].(userPermission); !ok {
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

	return &UserJwtPayload{
		ID:         id,
		Username:   username,
		Permission: permission,
	}, nil
}
