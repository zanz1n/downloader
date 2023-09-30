package server

import (
	"os"

	"github.com/valyala/fasthttp"
	"github.com/zanz1n/downloader/cmd/node/config"
	"github.com/zanz1n/downloader/internal/errors"
	"github.com/zanz1n/downloader/internal/logger"
)

func (s *Server) HandleProxiedGetFile(c *fasthttp.RequestCtx) {
	fileId, ok := c.UserValue("id").(string)
	if !ok {
		c.SetStatusCode(500)
		return
	}

	rnd := c.Request.URI().QueryArgs().Peek("rnd")

	if rnd == nil {
		s.HandleError(c, errors.ErrRndQueryNotProvided)
		return
	} else if len(rnd) < 2 {
		s.HandleError(c, errors.ErrRndQueryNotProvided)
		return
	}

	err := s.ExtractSignatureAuthorization(c, rnd)
	if err != nil {
		s.HandleError(c, err)
		return
	}

	cfg := config.GetConfig()

	file, err := os.Open(cfg.App.DataDir + "/" + fileId)
	if err != nil {
		logger.Error("Registered file '%s' could not be found in the server storage", fileId)
		c.SetStatusCode(500)
		return
	}

	fi, err := file.Stat()
	if err != nil {
		file.Close()
		logger.Error("Failed to pick file '%s' FS info", fileId)
		c.SetStatusCode(500)
		return
	}

	filesize := int(fi.Size())

	if int64(filesize) != fi.Size() {
		filesize = -1
	}

	c.Response.SetBodyStream(file, int(fi.Size()))
}
