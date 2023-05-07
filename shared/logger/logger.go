package logger

import (
	"fmt"
	"io"
	"log"
	"os"
	"time"

	"github.com/gofiber/fiber/v2"
)

func defaultLogger(out io.Writer, prefix ...string) *log.Logger {
	p := fmt.Sprintf("%v\t", os.Getpid())

	if len(prefix) >= 1 {
		p = prefix[0]
	}

	return log.New(
		out,
		p,
		log.Ldate|log.Ltime,
	)
}

func statusColor(code int, colors fiber.Colors) string {
	switch {
	case code >= fiber.StatusOK && code < fiber.StatusMultipleChoices:
		return colors.Green
	case code >= fiber.StatusMultipleChoices && code < fiber.StatusBadRequest:
		return colors.Blue
	case code >= fiber.StatusBadRequest && code < fiber.StatusInternalServerError:
		return colors.Yellow
	default:
		return colors.Red
	}
}

func methodColor(method string, colors fiber.Colors) string {
	switch method {
	case fiber.MethodGet:
		return colors.Cyan
	case fiber.MethodPost:
		return colors.Green
	case fiber.MethodPut:
		return colors.Yellow
	case fiber.MethodDelete:
		return colors.Red
	case fiber.MethodPatch:
		return colors.White
	case fiber.MethodHead:
		return colors.Magenta
	case fiber.MethodOptions:
		return colors.Blue
	default:
		return colors.Reset
	}
}

var (
	stdOutLogger *log.Logger = defaultLogger(os.Stdout)
	stdErrLogger *log.Logger = defaultLogger(os.Stderr)

	config *Config
)

type Config struct {
	InfoPrefix    string
	WarningPrefix string
	ErrorPrefix   string
	HttpPrefix    string
	Colors        bool
}

func Init() {
	config = &Config{}

	config.InfoPrefix = "\tINFO\t"
	config.ErrorPrefix = "\tERROR\n"
	config.HttpPrefix = "\tHTTP\t"
	config.WarningPrefix = "\tWARN\t"

	if os.Getenv("TERM") == "dumb" || os.Getenv("NO_COLOR") == "1" {
		config.Colors = false
	} else {
		config.Colors = true
		config.InfoPrefix = "\x1b[36m" + config.InfoPrefix + "\x1b[0m"
		config.WarningPrefix = "\x1b[33m" + config.WarningPrefix + "\x1b[0m"
		config.ErrorPrefix = "\x1b[31m" + config.ErrorPrefix + "\x1b[0m"
		config.HttpPrefix = "\x1b[34m" + config.HttpPrefix + "\x1b[0m"
	}
}

func Info(format string, args ...any) {
	stdOutLogger.Output(2, fmt.Sprintf(config.InfoPrefix+format+"\n", args...))
}

func Warn(format string, args ...any) {
	stdOutLogger.Output(2, fmt.Sprintf(config.WarningPrefix+format+"\n", args...))
}

func Error(format string, args ...any) {
	stdErrLogger.Output(2, fmt.Sprintf(config.ErrorPrefix+format+"\n", args...))
}

func Http(format string, args ...any) {
	stdOutLogger.Output(2, fmt.Sprintf(config.HttpPrefix+format, args...))
}

func Fatal(format string, args ...any) {
	Error(format, args...)
	os.Exit(1)
}

func NewFiberMiddleware() fiber.Handler {
	return func(c *fiber.Ctx) error {
		start := time.Now()

		c.Next()

		end := time.Now()

		if config.Colors {
			Http(
				"[%s]:%s  %s%s\x1b[0m  %s  %s%v\x1b[0m  %s%v\x1b[0m",
				c.IP(),
				c.Port(),
				methodColor(c.Method(), fiber.DefaultColors),
				c.Method(),
				c.Path(),
				statusColor(c.Response().StatusCode(), fiber.DefaultColors),
				c.Response().StatusCode(),
				"\x1b[90m",
				end.Sub(start),
			)
		} else {
			Http(
				"[%s]:%s  %s  %s  %v  %v",
				c.IP(),
				c.Port(),
				c.Method(),
				c.Path(),
				c.Response().StatusCode(),
				end.Sub(start),
			)
		}

		return nil
	}
}
