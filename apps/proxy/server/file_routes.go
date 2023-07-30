package server

import (
	"bytes"
	"context"
	"strconv"
	"time"

	"github.com/valyala/fasthttp"
	"github.com/zanz1n/downloader/shared/errors"
	"github.com/zanz1n/downloader/shared/logger"
	"github.com/zanz1n/downloader/shared/utils"
)

func (s *Server) HandleGetFile(c *fasthttp.RequestCtx) {
	fileId, ok := c.UserValue("id").(string)
	if !ok {
		c.SetStatusCode(500)
		return
	}

	perm, err := s.ExtractFileAuthorization(c, fileId)
	if err != nil {
		s.HandleError(c, err)
		return
	}
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	info, err := s.db.GetFileAndNodeInfo(ctx, fileId)
	if err != nil {
		s.HandleError(c, errors.ErrFileNotFound)
		return
	}

	if perm.IsUserToken && perm.UserID != info.UserId {
		s.HandleError(c, errors.ErrFileAccessDenied)
		return
	}

	req := fasthttp.AcquireRequest()
	defer fasthttp.ReleaseRequest(req)

	scheme := "http"
	if info.NodeSSL {
		scheme = "https"
	}

	rnd, err := randomString(24)
	if err != nil {
		s.HandleError(c, err)
		return
	}

	req.SetRequestURI(scheme + "://" + info.NodeAddress + ":" +
		strconv.Itoa(int(info.NodePort)) + "/file/" + info.ID + "?rnd=" + rnd)

	sig, err := generateSignature(utils.S2B(rnd))
	if err != nil {
		s.HandleError(c, err)
		return
	}

	req.Header.Add("Authorization", "Signature "+utils.B2S(sig))
	res := fasthttp.AcquireResponse()
	defer fasthttp.ReleaseResponse(res)

	if err := s.client.Do(req, res); err != nil {
		logger.Error("Failed to connect to '%s' node: "+err.Error(), info.NodeId)
		s.HandleError(c, errors.ErrFailedToFetchFileNode)
		return
	}

	if res.StatusCode() != 200 {
		logger.Error("Failed to to fetch file '%s' on node '%s': StatusCode %v",
			info.ID,
			info.NodeId,
			res.StatusCode())
		s.HandleError(c, errors.ErrFailedToFetchFileNode)
		return
	}

	c.Response.Header.Add("File-Checksum", info.Checksum)
	c.Response.Header.SetContentType(info.ContentType)

	filename := sanitizeFileName(info.Name, info.ContentType)

	c.Response.Header.Set(
		"Content-Disposition",
		"attachment; filename="+filename,
	)

	if res.IsBodyStream() {
		c.Response.SetBodyStream(res.BodyStream(), res.Header.ContentLength())
	} else {
		b := res.Body()
		c.Response.SetBodyStream(bytes.NewReader(b), len(b))
	}
}
