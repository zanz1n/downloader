package user

import (
	"context"
	"time"

	nanoid "github.com/matoous/go-nanoid/v2"
	"github.com/zanz1n/downloader/dba"
	"github.com/zanz1n/downloader/shared/auth"
	"github.com/zanz1n/downloader/shared/errors"
)

func NewUserService(db dba.Querier) *UserService {
	return &UserService{
		db: db,
	}
}

type UserService struct {
	db dba.Querier
}

func (s *UserService) CreateUser(role dba.UserRole, data *SignUpBody) (*dba.User, error) {
	if err := data.Validate(); err != nil {
		return nil, err
	}

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	id, err := nanoid.New(12)
	if err != nil {
		return nil, errors.ErrNanoIdGenerationFailed
	}

	password, err := auth.HashPassword(data.Password)
	if err != nil {
		return nil, err
	}

	params := dba.CreateUserParams{
		ID:        id,
		FirstName: data.FirstName,
		LastName:  data.LastName,
		Email:     data.Email,
		Role:      role,
		Password:  password,
	}

	user, err := s.db.CreateUser(ctx, &params)
	if err != nil {
		return nil, errors.ErrUserAlreadyExists
	}

	return user, nil
}
