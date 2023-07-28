package server

import (
	"os"

	"github.com/valyala/fasthttp"
	"github.com/zanz1n/downloader/apps/node/config"
	"github.com/zanz1n/downloader/shared/errors"
	"github.com/zanz1n/downloader/shared/logger"
)

func (s *Server) HandleProxiedGetFile(c *fasthttp.RequestCtx) {
	fileId, ok := c.UserValue("id").(string)
	if !ok {
		c.SetStatusCode(400)
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
		logger.Error("Registered file '%s' could not be found in the server storage")
		c.SetStatusCode(500)
		return
	}
	defer file.Close()

	fi, err := file.Stat()
	if err != nil {
		logger.Error("Failed to pick file '%s' FS info")
		c.SetStatusCode(500)
		return
	}

	c.Response.SetBodyStream(file, int(fi.Size()))
}

func (s *Server) HandleGetFile(c *fasthttp.RequestCtx) {
	fileId, ok := c.UserValue("id").(string)
	if !ok {
		c.SetStatusCode(400)
		return
	}

	info, err := s.ExtractFileReadAuthorization(c, fileId)

	if err != nil {
		s.HandleError(c, err)
		return
	}

	cfg := config.GetConfig()

	file, err := os.Open(cfg.App.DataDir + "/" + info.ID)

	if err != nil {
		logger.Error("Registered file '%s' could not be found in the server storage")
		c.SetStatusCode(500)
		return
	}
	defer file.Close()

	fi, err := file.Stat()
	if err != nil {
		logger.Error("Failed to pick file '%s' FS info")
		c.SetStatusCode(500)
		return
	}

	c.Response.Header.Add("File-Checksum", info.Checksum)
	c.Response.Header.SetContentType(info.ContentType)

	filename := sanitizeFileName(info.Name, info.ContentType)

	c.Response.Header.Set(
		"Content-Disposition",
		"attachment; filename="+filename,
	)

	c.Response.SetBodyStream(file, int(fi.Size()))
}
