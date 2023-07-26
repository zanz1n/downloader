package errors

import (
	"sync"
)

var (
	mpeMu = sync.RWMutex{}
)

func GetStatusErr(key error) StatusError {
	mpeMu.RLock()
	defer mpeMu.RUnlock()

	v, ok := mpe[key]

	if !ok {
		return &statusErrorImpl{
			code:     0,
			httpCode: 500,
			message:  "Unknown err: " + key.Error(),
		}
	}

	return v
}

type statusErrorImpl struct {
	code     uint
	httpCode int
	message  string
}

func (e *statusErrorImpl) Message() string {
	return e.message
}

func (e *statusErrorImpl) CustomCode() uint {
	return e.code
}

func (e *statusErrorImpl) HttpCode() int {
	return e.httpCode
}

type StatusError interface {
	Message() string
	CustomCode() uint
	HttpCode() int
}

type errorImpl struct {
	m string
}

func (e *errorImpl) Error() string {
	return e.m
}

func New(text string) error {
	return &errorImpl{m: text}
}
