package server

import (
	"errors"
	"log"
	"os"
	"time"

	"github.com/valyala/fasthttp"
	"github.com/zanz1n/downloader/shared/logger"
	"github.com/zanz1n/downloader/shared/utils"
)

var serverLogger = logger.NewLogger("server")

type Server struct {
	fhttp *fasthttp.Server
}

func NewServer() *Server {
	s := Server{
		fhttp: &fasthttp.Server{
			StreamRequestBody: true,
			CloseOnShutdown:   true,
		},
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
