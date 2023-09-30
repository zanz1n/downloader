package server

import (
	"crypto/tls"
	"os"
	"time"

	"github.com/fasthttp/router"
	"github.com/valyala/fasthttp"
	"github.com/zanz1n/downloader/cmd/proxy/repository/auth"
	"github.com/zanz1n/downloader/cmd/proxy/repository/user"
	"github.com/zanz1n/downloader/internal/dba"
	"github.com/zanz1n/downloader/internal/errors"
	"github.com/zanz1n/downloader/internal/logger"
	"github.com/zanz1n/downloader/internal/utils"
)

var serverLogger = logger.NewLogger("server")

type Server struct {
	r      *router.Router
	fhttp  *fasthttp.Server
	db     dba.Querier
	as     *auth.AuthService
	us     *user.UserService
	client *fasthttp.Client
}

func NewServer(db dba.Querier, as *auth.AuthService, us *user.UserService) *Server {
	fhttp := fasthttp.Server{
		StreamRequestBody: true,
		CloseOnShutdown:   true,
	}
	r := router.New()

	s := Server{
		fhttp: &fhttp,
		db:    db,
		as:    as,
		us:    us,
		client: &fasthttp.Client{
			TLSConfig: &tls.Config{
				InsecureSkipVerify: true,
			},
		},
	}
	r.MethodNotAllowed = s.HandleMethodNotAllowed
	r.NotFound = s.HandleNotFound
	s.r = r

	return &s
}

func (s *Server) wireRoutes() {
	s.fhttp.Handler = s.Handler

	s.r.GET("/files/{id}", s.HandleGetFile)
	s.r.POST("/auth/signin", s.HandleSignin)
	s.r.POST("/auth/signup", s.HandleSignup)
}

func (s *Server) Handler(ctx *fasthttp.RequestCtx) {
	startTime := time.Now()

	s.r.Handler(ctx)

	logger.LogRequest(&logger.RequestInfo{
		Addr:       ctx.RemoteAddr(),
		Method:     utils.B2S(ctx.Method()),
		Path:       utils.B2S(ctx.URI().Path()),
		StatusCode: ctx.Response.StatusCode(),
		Duration:   time.Since(startTime),
	})
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

	respondJson(c, errBody)
	c.SetStatusCode(st.HttpCode())
}

func (s *Server) HandleMethodNotAllowed(c *fasthttp.RequestCtx) {
	c.Response.SetStatusCode(405)
	c.SetContentType("application/json")
	c.Response.SetBodyString("{\"message\":\"Method not allowed\",\"errorCode\":4050}")
}

func (s *Server) HandleNotFound(c *fasthttp.RequestCtx) {
	c.Response.SetStatusCode(404)
	c.SetContentType("application/json")
	c.Response.SetBodyString("{\"message\":\"Not found\",\"errorCode\":4040}")
}

func (s *Server) MustListenAndServeTLS(addr, certPath, keyPath string) {
	if err := s.ListenAndServeTLS(addr, certPath, keyPath); err != nil {
		serverLogger.Fatal(err)
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
	s.wireRoutes()

	serverLogger.Info("Listening with tls for " + addr)

	return s.fhttp.ListenAndServeTLSEmbed(addr, certData, keyData)
}

func (s *Server) MustListenAndServe(addr string) {
	if err := s.ListenAndServe(addr); err != nil {
		serverLogger.Fatal(err)
	}
}

func (s *Server) ListenAndServe(addr string) error {
	s.wireRoutes()

	serverLogger.Info("Listening for " + addr)

	return s.fhttp.ListenAndServe(addr)
}
