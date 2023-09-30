package server

import (
	"github.com/goccy/go-json"
	"github.com/valyala/fasthttp"
	"github.com/zanz1n/downloader/cmd/proxy/config"
	"github.com/zanz1n/downloader/cmd/proxy/repository/user"
	"github.com/zanz1n/downloader/internal/dba"
	"github.com/zanz1n/downloader/internal/errors"
	"github.com/zanz1n/downloader/internal/logger"
)

func (s *Server) HandleSignin(c *fasthttp.RequestCtx) {
	if c.IsBodyStream() {
		s.HandleError(c, errors.ErrStreamNotSupported)
		return
	}

	payload := user.SignInBody{}
	if err := json.Unmarshal(c.Request.Body(), &payload); err != nil {
		logger.Debug(err.Error())
		s.HandleError(c, errors.ErrInvalidJson)
		return
	}

	if err := payload.Validate(); err != nil {
		s.HandleError(c, err)
		return
	}

	token, err := s.as.AuthUser(payload.Email, payload.Password)
	if err != nil {
		s.HandleError(c, err)
		return
	}

	c.SetStatusCode(201)
	respondJson(c, dataBody(token, "Success"))
}

func (s *Server) HandleSignup(c *fasthttp.RequestCtx) {
	cfg := config.GetConfig()

	if !cfg.AllowSignUp {
		s.HandleError(c, errors.ErrSignupNotAllowed)
		return
	}
	if c.IsBodyStream() {
		s.HandleError(c, errors.ErrStreamNotSupported)
		return
	}

	payload := user.SignUpBody{}
	if err := json.Unmarshal(c.Request.Body(), &payload); err != nil {
		s.HandleError(c, errors.ErrInvalidJson)
		return
	}

	if err := payload.Validate(); err != nil {
		s.HandleError(c, err)
		return
	}

	user, err := s.us.CreateUser(dba.UserRoleUSER, &payload)
	if err != nil {
		s.HandleError(c, err)
		return
	}

	c.SetStatusCode(201)
	respondJson(c, dataBody(user.ToApiUser(), "Success"))
}
