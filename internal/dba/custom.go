package dba

import (
	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgtype"
	"github.com/zanz1n/downloader/internal/errors"
)

func UUIDToString(id pgtype.UUID) string {
	return uuid.UUID(id.Bytes).String()
}

func UUIDFromString(s string) (pgtype.UUID, error) {
	u, err := uuid.Parse(s)
	if err != nil {
		return pgtype.UUID{
			Valid: false,
		}, errors.ErrInvalidUUID
	}

	return pgtype.UUID{
		Valid: true,
		Bytes: u,
	}, nil
}

func NewUUID() pgtype.UUID {
	return pgtype.UUID{
		Valid: true,
		Bytes: uuid.New(),
	}
}

type ApiUser struct {
	ID        uuid.UUID        `json:"id"`
	CreatedAt pgtype.Timestamp `json:"createdAt"`
	UpdatedAt pgtype.Timestamp `json:"updatedAt"`
	FirstName string           `json:"firstName"`
	LastName  string           `json:"lastName"`
	Email     string           `json:"email"`
	Deleted   bool             `json:"deleted"`
	Role      UserRole         `json:"role"`
}

func (u *User) ToApiUser() *ApiUser {
	return &ApiUser{
		ID:        u.ID.Bytes,
		CreatedAt: u.CreatedAt,
		UpdatedAt: u.UpdatedAt,
		FirstName: u.FirstName,
		LastName:  u.LastName,
		Email:     u.Email,
		Deleted:   u.Deleted,
		Role:      u.Role,
	}
}
