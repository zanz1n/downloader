package server

import (
	"github.com/goccy/go-json"
	"github.com/valyala/fasthttp"
	"github.com/zanz1n/downloader/dba"
	"github.com/zanz1n/downloader/proxy/config"
	"github.com/zanz1n/downloader/proxy/repository/user"
	"github.com/zanz1n/downloader/shared/errors"
)

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
	respondJson(c, user)
}
