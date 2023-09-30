package logger

import (
	"net"
	"strconv"
	"time"
)

const (
	reset     = "\x1b[0m"
	fgRed     = "\x1b[31m"
	fgGreen   = "\x1b[32m"
	fgYellow  = "\x1b[33m"
	fgBlue    = "\x1b[34m"
	fgMagenta = "\x1b[35m"
	fgCyan    = "\x1b[36m"
	fgGray    = "\x1b[90m"
)

type RequestInfo struct {
	Addr       net.Addr
	Method     string
	Path       string
	StatusCode int
	Duration   time.Duration
}

var httpLogger Logger

func LogRequest(info *RequestInfo) {
	if DefaultConfig.Colors {
		httpLogger.Info(
			"%s %s %s %s "+fgGray+"%v"+reset,
			info.Addr.String(),
			methodColor(info.Method),
			info.Path,
			statusColor(info.StatusCode),
			info.Duration,
		)
		return
	}

	httpLogger.Info(info.Addr.String() + " " + info.Method + " " + info.Path +
		" " + strconv.Itoa(info.StatusCode) + info.Duration.String(),
	)
}

func methodColor(method string) string {
	switch method {
	case "GET":
		return fgCyan + method + reset
	case "POST":
		return fgGreen + method + reset
	case "PUT":
		return fgYellow + method + reset
	case "DELETE":
		return fgRed + method + reset
	case "PATCH":
		return fgYellow + method + reset
	case "HEAD":
		return fgMagenta + method + reset
	case "OPTIONS":
		return fgBlue + method + reset
	default:
		return method
	}
}

func statusColor(code int) string {
	cstr := strconv.Itoa(code)

	if code >= 100 && code < 200 {
		return fgMagenta + cstr + reset
	} else if code >= 200 && code < 300 {
		return fgGreen + cstr + reset
	} else if code >= 300 && code < 400 {
		return fgBlue + cstr + reset
	} else if code >= 400 && code < 500 {
		return fgYellow + cstr + reset
	} else {
		return fgRed + cstr + reset
	}
}
