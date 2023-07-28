package server

import (
	"mime"
	"os"

	"github.com/valyala/fasthttp"
	"github.com/zanz1n/downloader/apps/node/config"
	"github.com/zanz1n/downloader/shared/logger"
)

func sanitizeFileName(name string, contentType string) string {
	foundDot := false
	for i := len(name); i != 0; i-- {
		if name[i] == '.' {
			foundDot = true
			break
		}
	}

	if foundDot {
		return name
	}

	exts, err := mime.ExtensionsByType(contentType)

	if err == nil && exts != nil {
		if len(exts) > 0 {
			ext := exts[0]

			if len(ext) > 0 {
				if ext[0] != '.' {
					ext = "." + ext
				}
			}

			name = name + ext
		}
	}
	return name
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
