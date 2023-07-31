package server

import (
	"bytes"
	"context"
	"crypto/tls"
	"io"
	"net"
	"strconv"
	"time"

	"github.com/valyala/fasthttp"
	"github.com/zanz1n/downloader/dba"
	"github.com/zanz1n/downloader/shared/errors"
	"github.com/zanz1n/downloader/shared/logger"
	"github.com/zanz1n/downloader/shared/transport"
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

	var reader io.Reader

	if info.NodeTCPPort.Valid {
		reader, err = s.handleGetFileTCP(c, info)
	} else {
		reader, err = s.handleGetFileHttp(c, info)
	}

	if err != nil {
		s.HandleError(c, err)
		return
	}

	c.Response.Header.Add("File-Checksum", info.Checksum)
	c.Response.Header.SetContentType(info.ContentType)

	filename := sanitizeFileName(info.Name, info.ContentType)

	c.Response.Header.Set(
		"Content-Disposition",
		"attachment; filename="+filename,
	)

	c.Response.SetBodyStream(reader, -1)
}

func (s *Server) handleGetFileTCP(c *fasthttp.RequestCtx, info *dba.GetFileAndNodeInfoRow) (io.Reader, error) {
	rnd, err := randomString(24)
	if err != nil {
		return nil, err
	}

	sig, err := generateSignature(utils.S2B(rnd))
	if err != nil {
		return nil, err
	}

	idenInfo := transport.IdenPayload{
		ID:     info.ID,
		Random: rnd,
		Token:  utils.B2S(sig),
		Type:   transport.RequestTypeRead,
	}

	idenBuf, err := idenInfo.Encode()
	if err != nil {
		return nil, err
	}

	var (
		addr = info.NodeAddress + ":" + strconv.Itoa(int(info.NodeTCPPort.Int32))
		conn net.Conn
	)

	if info.NodeSSL {
		conn, err = tls.Dial("tcp", addr, &tls.Config{
			InsecureSkipVerify: true,
		})
	} else {
		conn, err = net.Dial("tcp", addr)
	}

	if err != nil {
		return nil, errors.ErrFailedToFetchFileNode
	}

	if _, err = conn.Write(idenBuf); err != nil {
		return nil, errors.ErrFailedToFetchFileNode
	}

	return conn, nil
}

func (s *Server) handleGetFileHttp(c *fasthttp.RequestCtx, info *dba.GetFileAndNodeInfoRow) (io.Reader, error) {
	req := fasthttp.AcquireRequest()
	defer fasthttp.ReleaseRequest(req)

	scheme := "http"
	if info.NodeSSL {
		scheme = "https"
	}

	rnd, err := randomString(24)
	if err != nil {
		return nil, err
	}

	req.SetRequestURI(scheme + "://" + info.NodeAddress + ":" +
		strconv.Itoa(int(info.NodePort)) + "/file/" + info.ID + "?rnd=" + rnd)

	sig, err := generateSignature(utils.S2B(rnd))
	if err != nil {
		return nil, err
	}

	req.Header.Add("Authorization", "Signature "+utils.B2S(sig))
	res := fasthttp.AcquireResponse()
	defer fasthttp.ReleaseResponse(res)

	if err := s.client.Do(req, res); err != nil {
		logger.Error("Failed to connect to '%s' node: "+err.Error(), info.NodeId)
		return nil, err
	}

	if res.StatusCode() != 200 {
		logger.Error("Failed to to fetch file '%s' on node '%s': StatusCode %v",
			info.ID,
			info.NodeId,
			res.StatusCode())
		return nil, err
	}

	if res.IsBodyStream() {
		return res.BodyStream(), nil
	} else {
		b := res.Body()
		return bytes.NewReader(b), nil
	}
}
