package apperror

import "net/http"

type Error struct {
	Status  int
	Message string
	Err     error
}

func (e *Error) Error() string {
	if e.Err != nil {
		return e.Message + ": " + e.Err.Error()
	}
	return e.Message
}

func (e *Error) Unwrap() error {
	return e.Err
}

func New(status int, message string, err ...error) *Error {
	e := &Error{
		Status:  status,
		Message: message,
	}
	if len(err) > 0 {
		e.Err = err[0]
	}
	return e
}

func BadRequest(message string, err ...error) *Error {
	return New(http.StatusBadRequest, message, err...)
}

func Unauthorized(message string, err ...error) *Error {
	return New(http.StatusUnauthorized, message, err...)
}

func Forbidden(message string, err ...error) *Error {
	return New(http.StatusForbidden, message, err...)
}

func NotFound(message string, err ...error) *Error {
	return New(http.StatusNotFound, message, err...)
}

func Conflict(message string, err ...error) *Error {
	return New(http.StatusConflict, message, err...)
}

func Internal(message string, err ...error) *Error {
	return New(http.StatusInternalServerError, message, err...)
}
