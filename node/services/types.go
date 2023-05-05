package services

type AuthRejectErr interface {
	Error() string
	Status() int
}

type authRejectErrImpl struct {
	err    string
	status int
}

func (ar *authRejectErrImpl) Error() string {
	return ar.err
}

func (ar *authRejectErrImpl) Status() int {
	return ar.status
}

func NewAuthRejectErr(msg string, status int) AuthRejectErr {
	err := new(authRejectErrImpl)

	err.err = msg
	err.status = status

	return err
}
