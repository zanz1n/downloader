package logger

import (
	"fmt"
	"os"
	"strconv"
	"time"
)

type Logger interface {
	Debug(format string, args ...any)
	Info(format string, args ...any)
	Warn(format string, args ...any)
	Error(format string, args ...any)
	Fatal(...any)
}

func NewLogger(target string, cfg ...*Config) Logger {
	l := LoggerImpl{
		out:    os.Stdout,
		target: target,
	}

	if target != "" {
		l.target = "  " + target
	}

	if len(cfg) == 0 {
		l.cfg = DefaultConfig
	} else {
		l.cfg = cfg[0]
	}

	return &l
}

type LoggerImpl struct {
	out    *os.File
	target string
	cfg    *Config
}

func (l *LoggerImpl) Debug(format string, args ...any) {
	if !level.WillLog(LoggerLevelDebug) {
		return
	}
	fm := fmt.Sprintf(format, args...)

	l.out.WriteString("\r" +
		l.cfg.SBL + nowFormated() +
		l.cfg.DebugPrefix + l.target +
		l.cfg.SBR + fm + "\n",
	)
}

func (l *LoggerImpl) Info(format string, args ...any) {
	if !level.WillLog(LoggerLevelInfo) {
		return
	}
	fm := fmt.Sprintf(format, args...)

	l.out.WriteString("\r" +
		l.cfg.SBL + nowFormated() +
		l.cfg.InfoPrefix + l.target +
		l.cfg.SBR + fm + "\n",
	)
}

func (l *LoggerImpl) Warn(format string, args ...any) {
	if !level.WillLog(LoggerLevelWarn) {
		return
	}
	fm := fmt.Sprintf(format, args...)

	l.out.WriteString("\r" +
		l.cfg.SBL + nowFormated() +
		l.cfg.WarningPrefix + l.target +
		l.cfg.SBR + fm + "\n",
	)
}

func (l *LoggerImpl) Error(format string, args ...any) {
	if !level.WillLog(LoggerLevelError) {
		return
	}
	fm := fmt.Sprintf(format, args...)

	l.out.WriteString("\r" +
		l.cfg.SBL + nowFormated() +
		l.cfg.ErrorPrefix + l.target +
		l.cfg.SBR + fm + "\n",
	)
}

func (l *LoggerImpl) Fatal(args ...any) {
	var fm string
	if len(args) > 0 {
		if s, ok := args[0].(string); ok {
			fm = fmt.Sprintf(s, args[1:]...)
		} else {
			fm = fmt.Sprint(args...)
		}
	} else {
		fm = fmt.Sprint(args...)
	}

	l.out.WriteString("\r" +
		l.cfg.SBL + nowFormated() +
		l.cfg.ErrorPrefix + l.target +
		l.cfg.SBR + fm + "\n",
	)
	os.Exit(1)
}

var (
	DefaultConfig *Config
	level         *LoggerLevel
	defaultLogger Logger
)

type Config struct {
	InfoPrefix    string
	WarningPrefix string
	ErrorPrefix   string
	DebugPrefix   string
	Colors        bool
	SBL, SBR      string
}

func init() {
	DefaultConfig = &Config{}

	DefaultConfig.InfoPrefix = " INFO"
	DefaultConfig.ErrorPrefix = " ERROR"
	DefaultConfig.WarningPrefix = " WARN"
	DefaultConfig.DebugPrefix = " DEBUG"

	if os.Getenv("TERM") == "dumb" || os.Getenv("NO_COLOR") == "1" {
		DefaultConfig.Colors = false

		DefaultConfig.SBR = "] "
		DefaultConfig.SBL = "["
	} else {
		DefaultConfig.SBR = "\x1b[90m]\x1b[0m "
		DefaultConfig.SBL = "\x1b[90m[\x1b[0m"

		DefaultConfig.Colors = true
		DefaultConfig.InfoPrefix = "\x1b[32m" + DefaultConfig.InfoPrefix + "\x1b[0m"
		DefaultConfig.WarningPrefix = "\x1b[33m" + DefaultConfig.WarningPrefix + "\x1b[0m"
		DefaultConfig.ErrorPrefix = "\x1b[31m" + DefaultConfig.ErrorPrefix + "\x1b[0m"
		DefaultConfig.DebugPrefix = "\x1b[36m" + DefaultConfig.DebugPrefix + "\x1b[0m"
	}

	SetLevel(os.Getenv("LOGGER_LEVEL"))

	defaultLogger = &LoggerImpl{
		out:    os.Stdout,
		target: "",
		cfg:    DefaultConfig,
	}

	httpLogger = NewLogger("http_log")
}

func SetLevel(t string) {
	var target LoggerLevel
	switch t {
	case "info":
		target = LoggerLevelInfo
	case "3":
		target = LoggerLevelInfo
	case "warn":
		target = LoggerLevelWarn
	case "2":
		target = LoggerLevelWarn
	case "error":
		target = LoggerLevelError
	case "1":
		target = LoggerLevelError
	default:
		target = LoggerLevelAll
	}

	if level == nil {
		level = &target
	} else {
		*level = target
	}

}

type LoggerLevel uint8

const (
	LoggerLevelDebug LoggerLevel = 4
	LoggerLevelInfo  LoggerLevel = 3
	LoggerLevelWarn  LoggerLevel = 2
	LoggerLevelError LoggerLevel = 1
	LoggerLevelAll   LoggerLevel = 0
)

func (cfg *LoggerLevel) WillLog(target LoggerLevel) bool {
	if *cfg == 0 {
		return true
	}

	if target > *cfg {
		return false
	} else {
		return true
	}
}

func left10(n int) string {
	ns := strconv.Itoa(n)

	if n < 10 {
		ns = "0" + ns
	}
	return ns
}

func nowFormated() string {
	now := time.Now()

	ms := now.Nanosecond() / 1000
	mss := strconv.Itoa(ms)

	switch {
	case ms < 10:
		mss = "00000" + mss
	case ms < 100:
		mss = "0000" + mss
	case ms < 1000:
		mss = "000" + mss
	case ms < 10000:
		mss = "00" + mss
	case ms < 100000:
		mss = "0" + mss
	}

	return strconv.Itoa(now.Year()) + "/" +
		left10(int(now.Month())) + "/" +
		left10(now.Day()) + " " +
		left10(now.Hour()) + ":" +
		left10(now.Minute()) + ":" +
		left10(now.Second()) + "." +
		mss
}

func Debug(format string, args ...any) {
	defaultLogger.Debug(format, args...)
}

func Info(format string, args ...any) {
	defaultLogger.Info(format, args...)
}

func Warn(format string, args ...any) {
	defaultLogger.Warn(format, args...)
}

func Error(format string, args ...any) {
	defaultLogger.Error(format, args...)
}

func Fatal(args ...any) {
	defaultLogger.Fatal(args...)
}
