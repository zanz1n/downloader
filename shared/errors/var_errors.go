package errors

var mpe = map[error]StatusError{
	ErrPasswordHashingFailed: statusInternalServerError,
	ErrUserAuthFailed:        statusUserAuthFailed,
	TokenGenerationFailed:    statusInternalServerError,
}

var (
	statusInternalServerError = &statusErrorImpl{
		code:     5000,
		httpCode: 500,
		message:  "Internal server error",
	}
	statusUserNotFound = &statusErrorImpl{
		code:     4041,
		httpCode: 404,
		message:  "User could not be found",
	}
	statusUserAuthFailed = &statusErrorImpl{
		code:     4011,
		httpCode: 401,
		message:  "User not found or password do not match",
	}
)

var (
	TokenGenerationFailed    = New("failed to generate the jwt token")
	ErrPasswordHashingFailed = New("failed to hash user password")
	ErrUserAuthFailed        = New("user not found or password do not match")
)
