package auth_test

import (
	"crypto/ed25519"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/zanz1n/downloader/cmd/proxy/repository/auth"
	"github.com/zanz1n/downloader/internal/dba"
)

func mockQuerier() dba.Querier {
	return nil
}

func mockAuthService() (*auth.AuthService, error) {
	pubKey, privKey, err := ed25519.GenerateKey(nil)

	if err != nil {
		return nil, err
	}

	hmacKey := []byte("y0/t64x1xItpbuUyksYp62IS/mt64sNHfdVuaA==")

	as := auth.NewAuthService(mockQuerier(), &auth.Options{
		UserTokenDuration: time.Second * 3,
		JwtEdDSAPrivKey:   privKey,
		JwtEdDSAPubKey:    pubKey,
		JwtHmacKey:        hmacKey,
	})

	return as, nil
}

func TestFileAccessToken(t *testing.T) {
	as, err := mockAuthService()

	if err != nil {
		t.Fatal("Failed to mock auth service: " + err.Error())
	}

	fileId := uuid.NewString()

	token, err := as.CreateFileAccessToken(fileId, auth.FileAccessPermRead, 3*time.Second)

	if err != nil {
		t.Fatal("Failed to create file access token: " + err.Error())
	}

	t.Log("File acess token created successfully")

	claims, err := as.DecodeFileAccessToken(token)

	if err != nil {
		t.Fatal("Failed to decode created token: " + err.Error())
	}

	t.Log("File access token decoded successfully")

	if claims.FileID != fileId || claims.Permission != auth.FileAccessPermRead {
		t.Fatal("Created file access token has inconsistent props")
	}

	time.Sleep(4 * time.Second)

	if _, err = as.DecodeFileAccessToken(token); err == nil {
		t.Fatal("File access token must be expired")
	}

	t.Log("File access token expired as expected")
}

func TestUserToken(t *testing.T) {
	as, err := mockAuthService()

	if err != nil {
		t.Fatal("Failed to mock auth service: " + err.Error())
	}

	userId := uuid.NewString()
	email := "test@example.com"
	now := time.Now()

	token, err := as.EncodeUserToken(&auth.UserJwtPayload{
		UserID:     userId,
		Email:      email,
		ExpiryDate: now.Add(as.UserTokenDuration).Unix(),
		IssuedAt:   now.Unix(),
		Role:       dba.UserRoleUSER,
	})

	if err != nil {
		t.Fatal("Failed to create user token: " + err.Error())
	}

	t.Log("User token created successfully")

	claims, err := as.DecodeUserToken(token)

	if err != nil {
		t.Fatal("Failed to decode created token: " + err.Error())
	}

	t.Log("User token decoded successfully")

	if claims.UserID != userId || claims.Email != email || claims.Role != dba.UserRoleUSER {
		t.Fatal("Created user token has inconsistent props")

	}

	time.Sleep(4 * time.Second)

	if _, err = as.DecodeUserToken(token); err == nil {
		t.Fatal("User token must be expired")
	}

	t.Log("User token expired as expected")
}
