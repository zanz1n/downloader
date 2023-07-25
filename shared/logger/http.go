package logger

import (
	"net"
	"strconv"
	"time"
)

type RequestInfo struct {
	Addr       net.Addr
	Method     string
	Path       string
	StatusCode int
	Duration   time.Duration
}

var httpLogger Logger

func init() {
	httpLogger = NewLogger("http_log")
}

func LogRequest(info *RequestInfo) {
	if DefaultConfig.Colors {
		httpLogger.Info(
			"%s  %s\x1b[0m  %s  %s%v\x1b[0m  %s%v\x1b[0m",
			info.Addr.String(),
			methodColor(info.Method)+info.Method,
			info.Path,
			statusColor(info.StatusCode),
			info.StatusCode,
			"\x1b[90m", // Gray
			info.Duration,
		)
		return
	}

	httpLogger.Info(info.Addr.String() + "  " + info.Method + "  " + info.Path +
		"  " + strconv.Itoa(info.StatusCode) + info.Duration.String(),
	)
}

func methodColor(method string) string {
	switch method {
	case "GET":
		return "\x1b[36m" // Cyan
	case "POST":
		return "\x1b[32m" // Green
	case "PUT":
		return "\x1b[33m" // Yellow
	case "DELETE":
		return "\x1b[31m" // Red
	case "PATCH":
		return "\x1b[33m" // Yellow
	case "HEAD":
		return "\x1b[35m" // Magenta
	case "OPTIONS":
		return "\x1b[34m" // Magenta
	default:
		return ""
	}
}

func statusColor(code int) string {
	if code >= 100 && code < 200 {
		return "\x1b[35m" // Magenta
	} else if code >= 200 && code < 300 {
		return "\x1b[32m" // Green
	} else if code >= 300 && code < 400 {
		return "\x1b[34m" // Blue
	} else if code >= 400 && code < 500 {
		return "\x1b[33m" // Yellow
	} else {
		return "\x1b[31m" // Red
	}
}
