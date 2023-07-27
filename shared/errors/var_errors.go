package errors

var mpe = map[error]StatusError{
	ErrPasswordHashingFailed: statusInternalServerError,
	ErrUserAuthFailed:        statusUserAuthFailed,
	ErrTokenGenerationFailed: statusInternalServerError,
	ErrRouteRequiresAuth:     statusRouteRequiresAuth,
	ErrExpiredJwtToken:       statusExpiredJwtToken,
	ErrInvalidJwtToken:       statusInvalidJwtToken,
	ErrDecodeTokenUnknownErr: statusInternalServerError,
	ErrInvalidAuthHeader:     statusInvalidAuthHeader,
	ErrFileAccessDenied:      statusFileAccessDenied,
	ErrFileNotFound:          statusFileNotFound,
}

var (
	statusInternalServerError = &statusErrorImpl{
		code:     5000,
		httpCode: 500,
		message:  "Something went wrong while processing your request",
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
	statusRouteRequiresAuth = &statusErrorImpl{
		code:     4012,
		httpCode: 401,
		message:  "This route requires authorization",
	}
	statusExpiredJwtToken = &statusErrorImpl{
		code:     4013,
		httpCode: 401,
		message:  "The provided authorization token is expired",
	}
	statusInvalidJwtToken = &statusErrorImpl{
		code:     4014,
		httpCode: 401,
		message:  "The provided authorization token is invalid",
	}
	statusInvalidAuthHeader = &statusErrorImpl{
		code:     4015,
		httpCode: 401,
		message:  "The provided authorization header is not valid",
	}
	statusFileAccessDenied = &statusErrorImpl{
		code:     4016,
		httpCode: 401,
		message:  "You do not have permission to access this file",
	}
	statusFileNotFound = &statusErrorImpl{
		code:     4042,
		httpCode: 404,
		message:  "The requested file could not be found",
	}
)

var (
	ErrRouteRequiresAuth     = New("this route requires authorization")
	ErrTokenGenerationFailed = New("failed to generate the jwt token")
	ErrPasswordHashingFailed = New("failed to hash user password")
	ErrUserAuthFailed        = New("user not found or password do not match")
	ErrInvalidJwtToken       = New("the provided authorization token is invalid")
	ErrExpiredJwtToken       = New("the provided authorization token is expired")
	ErrDecodeTokenUnknownErr = New("something went wrong while decoding the token")
	ErrInvalidAuthHeader     = New("the provided authorization header is not valid")
	ErrFileAccessDenied      = New("file access denied")
	ErrFileNotFound          = New("file could not be found")
)
