package errors

var mpe = map[error]StatusError{
	ErrPasswordHashingFailed:  statusInternalServerError,
	ErrUserAuthFailed:         statusUserAuthFailed,
	ErrTokenGenerationFailed:  statusInternalServerError,
	ErrRouteRequiresAuth:      statusRouteRequiresAuth,
	ErrExpiredJwtToken:        statusExpiredJwtToken,
	ErrInvalidJwtToken:        statusInvalidJwtToken,
	ErrDecodeTokenUnknownErr:  statusInternalServerError,
	ErrInvalidAuthHeader:      statusInvalidAuthHeader,
	ErrFileAccessDenied:       statusFileAccessDenied,
	ErrFileNotFound:           statusFileNotFound,
	ErrFileNotLocatedInNode:   statusFileNotLocatedInNode,
	ErrInvalidEmailProvided:   statusInvalidEmailProvided,
	ErrInvalidSignUpPayload:   statusInvalidSignUpBody,
	ErrUserAlreadyExists:      statusUserAlreadyExists,
	ErrEmailTooLarge:          statusEmailTooLarge,
	ErrPasswordWeak:           statusPasswordWeak,
	ErrUsernameTooLarge:       statusUsernameTooLarge,
	ErrNanoIdGenerationFailed: statusInternalServerError,
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
	statusFileNotLocatedInNode = &statusErrorImpl{
		code:     4043,
		httpCode: 404,
		message:  "The requested file is not located in this node",
	}
	statusInvalidEmailProvided = &statusErrorImpl{
		code:     4001,
		httpCode: 400,
		message:  "The provided email address is not valid",
	}
	statusInvalidSignUpBody = &statusErrorImpl{
		code:     4002,
		httpCode: 400,
		message:  "The provided sign up body is not valid",
	}
	statusUserAlreadyExists = &statusErrorImpl{
		code:     4091,
		httpCode: 409,
		message:  "This user already exists, maybe try a different email address",
	}
	statusEmailTooLarge = &statusErrorImpl{
		code:     4003,
		httpCode: 400,
		message:  "The provided email address is too big",
	}
	statusPasswordWeak = &statusErrorImpl{
		code:     4004,
		httpCode: 400,
		message:  "The provided password can not be used, too weak",
	}
	statusUsernameTooLarge = &statusErrorImpl{
		code:     4005,
		httpCode: 400,
		message:  "The provided username is too large to be used",
	}
)

var (
	ErrRouteRequiresAuth      = New("this route requires authorization")
	ErrTokenGenerationFailed  = New("failed to generate the jwt token")
	ErrPasswordHashingFailed  = New("failed to hash user password")
	ErrUserAuthFailed         = New("user not found or password do not match")
	ErrInvalidJwtToken        = New("the provided authorization token is invalid")
	ErrExpiredJwtToken        = New("the provided authorization token is expired")
	ErrDecodeTokenUnknownErr  = New("something went wrong while decoding the token")
	ErrInvalidAuthHeader      = New("the provided authorization header is not valid")
	ErrFileAccessDenied       = New("file access denied")
	ErrFileNotFound           = New("file could not be found")
	ErrFileNotLocatedInNode   = New("file is not located in this node")
	ErrInvalidEmailProvided   = New("the provided email address is not valid")
	ErrInvalidSignUpPayload   = New("the provided sign up body is not valid")
	ErrUserAlreadyExists      = New("user already exists")
	ErrEmailTooLarge          = New("the provided email address is too big")
	ErrPasswordWeak           = New("the provided password can not be used, too weak")
	ErrUsernameTooLarge       = New("the provided username is too large to be used")
	ErrNanoIdGenerationFailed = New("failed to generate nanoid")
)
