package server

import (
	"log"
	"os"
	"time"

	"github.com/valyala/fasthttp"
	"github.com/zanz1n/downloader/dba"
	"github.com/zanz1n/downloader/shared/auth"
	"github.com/zanz1n/downloader/shared/errors"
	"github.com/zanz1n/downloader/shared/logger"
	"github.com/zanz1n/downloader/shared/utils"
)

var serverLogger = logger.NewLogger("server")

type Server struct {
	fhttp *fasthttp.Server
	db    dba.Querier
	as    *auth.AuthService
}

func NewServer(db dba.Querier, as *auth.AuthService) *Server {
	fhttp := fasthttp.Server{
		StreamRequestBody: true,
		CloseOnShutdown:   true,
	}

	s := Server{
		fhttp: &fhttp,
		db:    db,
		as:    as,
	}

	return &s
}

func (s *Server) Handler(ctx *fasthttp.RequestCtx) {
	startTime := time.Now()

	ctx.SetStatusCode(404)

	logger.LogRequest(&logger.RequestInfo{
		Addr:       ctx.RemoteAddr(),
		Method:     utils.B2S(ctx.Method()),
		Path:       utils.B2S(ctx.URI().Path()),
		StatusCode: ctx.Response.StatusCode(),
		Duration:   time.Since(startTime),
	})

	log.Println(time.Since(startTime))
}

func (s *Server) Shutdown() {
	serverLogger.Info("Shutting down...")
	s.fhttp.Shutdown()
}

func (s *Server) HandleError(c *fasthttp.RequestCtx, err error) {
	st := errors.GetStatusErr(err)

	errBody := errors.ErrorBody{
		Message:   st.Message(),
		ErrorCode: st.CustomCode(),
	}

	c.SetBody(errBody.Marshal())
	c.SetStatusCode(st.HttpCode())
}

func (s *Server) MustListenAndServeTLS(addr, certPath, keyPath string) {
	if err := s.ListenAndServeTLS(addr, certPath, keyPath); err != nil {
		logger.Fatal(err)
	}
}

func (s *Server) ListenAndServeTLS(addr, certPath, keyPath string) error {
	certData, err := os.ReadFile(certPath)

	if err != nil {
		return errors.New("failed to open ssl certificate at " + certPath)
	}

	keyData, err := os.ReadFile(keyPath)

	if err != nil {
		return errors.New("failed to open ssl key at " + certPath)
	}

	s.fhttp.Handler = s.Handler

	serverLogger.Info("Listening with tls for " + addr)

	return s.fhttp.ListenAndServeTLSEmbed(addr, certData, keyData)
}

func (s *Server) MustListenAndServe(addr string) {
	if err := s.ListenAndServe(addr); err != nil {
		logger.Fatal(err)
	}
}

func (s *Server) ListenAndServe(addr string) error {
	s.fhttp.Handler = s.Handler

	serverLogger.Info("Listening for " + addr)

	return s.fhttp.ListenAndServe(addr)
}
