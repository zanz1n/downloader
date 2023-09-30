package user

import "github.com/zanz1n/downloader/internal/errors"

type SignInBody struct {
	Email    string `json:"email,omitempty" validate:"required"`
	Password string `json:"password,omitempty" validate:"required"`
}

func (p *SignInBody) Validate() error {
	if err := validate.Struct(p); err != nil {
		return errors.ErrInvalidSignInPayload
	}

	if !mailRegex.MatchString(p.Email) {
		return errors.ErrInvalidEmailProvided
	}

	return nil
}

type SignUpBody struct {
	FirstName string `json:"firstName,omitempty" validate:"required"`
	LastName  string `json:"lastName,omitempty" validate:"required"`
	Email     string `json:"email,omitempty" validate:"required"`
	Password  string `json:"password,omitempty" validate:"required"`
}

func (p *SignUpBody) Validate() error {
	if err := validate.Struct(p); err != nil {
		return errors.ErrInvalidSignUpPayload
	}

	switch {
	case !mailRegex.MatchString(p.Email):
		return errors.ErrInvalidEmailProvided
	case len(p.FirstName) > 16 || len(p.LastName) > 24:
		return errors.ErrUsernameTooLarge
	case len(p.Email) > 64:
		return errors.ErrEmailTooLarge
	case IsWeakPassword(p.Password):
		return errors.ErrPasswordWeak
	}

	return nil
}
