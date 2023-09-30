package server

import (
	"crypto/rand"
	"crypto/sha256"
	"encoding/hex"
	"mime"

	"github.com/goccy/go-json"
	"github.com/valyala/fasthttp"
	"github.com/zanz1n/downloader/cmd/proxy/config"
	"github.com/zanz1n/downloader/internal/errors"
	"github.com/zanz1n/downloader/internal/logger"
	"github.com/zanz1n/downloader/internal/utils"
)

func sanitizeFileName(name string, contentType string) string {
	foundDot := false
	if len(name) > 2 {
		for i := len(name) - 1; i > 0; i-- {
			if name[i] == '.' {
				foundDot = true
				break
			}
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
		logger.Error("Failed to marshal request body: " + err.Error())
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

	buf := hash.Sum([]byte{})
	hexBuf := make([]byte, len(buf)*2)

	hex.Encode(hexBuf, buf)

	return hexBuf, nil
}

func randomString(l int) (string, error) {
	b := make([]byte, l)

	if _, err := rand.Read(b); err != nil {
		logger.Error("Failed to read os random reader :" + err.Error())
		return "", errors.ErrHashingFailed
	}
	bo := make([]byte, len(b)*2)

	hex.Encode(bo, b)

	return utils.B2S(bo), nil
}
