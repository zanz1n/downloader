package dba

import "github.com/jackc/pgx/v5/pgtype"

type ApiUser struct {
	ID        string           `json:"id"`
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
		ID:        u.ID,
		CreatedAt: u.CreatedAt,
		UpdatedAt: u.UpdatedAt,
		FirstName: u.FirstName,
		LastName:  u.LastName,
		Email:     u.Email,
		Deleted:   u.Deleted,
		Role:      u.Role,
	}
}
