package server

import (
	"crypto/rand"
	"crypto/sha256"
	"encoding/base64"
	"mime"

	"github.com/goccy/go-json"
	"github.com/valyala/fasthttp"
	"github.com/zanz1n/downloader/proxy/config"
	"github.com/zanz1n/downloader/shared/errors"
	"github.com/zanz1n/downloader/shared/logger"
	"github.com/zanz1n/downloader/shared/utils"
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

func respondJson(c *fasthttp.RequestCtx, v any) {
	c.SetContentType("application/json")

	buf, err := json.Marshal(v)
	if err != nil {
		c.SetStatusCode(500)
		c.SetBody([]byte("{\"message\":\"Failed to marshal response body\",\"errorCode\":5000}"))
		return
	}

	c.SetBody(buf)
}

func generateSignature(rnd []byte) ([]byte, error) {
	hash := sha256.New()

	if _, err := hash.Write(rnd); err != nil {
		logger.Error("Hashing failed: " + err.Error())
		return nil, errors.ErrHashingFailed
	}
	if _, err := hash.Write(utils.S2B(config.GetConfig().Key)); err != nil {
		logger.Error("Hashing failed: " + err.Error())
		return nil, errors.ErrHashingFailed
	}

	base64Buf := []byte{}

	base64.StdEncoding.Encode(base64Buf, hash.Sum([]byte{}))

	return base64Buf, nil
}

func randomString(l int) (string, error) {
	b := make([]byte, l)

	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	bo := []byte{}

	base64.RawStdEncoding.Encode(bo, b)

	return utils.B2S(bo), nil
}
