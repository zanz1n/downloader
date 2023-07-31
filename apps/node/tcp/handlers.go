package tcp

import (
	"io"
	"net"
	"os"

	"github.com/zanz1n/downloader/node/config"
	"github.com/zanz1n/downloader/shared/logger"
	"github.com/zanz1n/downloader/shared/transport"
)

func (s *Server) HandleRead(conn net.Conn, iden *transport.IdenPayload) {
	file, err := os.Open(config.GetConfig().App.DataDir + "/" + iden.ID)
	if err != nil {
		logger.Error("Registered file '%s' could not be found in the server storage", iden.ID)
		return
	}

	io.Copy(conn, file)
}

func (s *Server) HandleWrite(conn net.Conn, iden *transport.IdenPayload) {
	cfg := config.GetConfig()

	file, err := os.Create(cfg.App.DataDir + "/" + iden.ID)
	if err != nil {
		logger.Error("Failed to create file '%s': "+err.Error(), iden.ID)
		return
	}

	if _, err = io.Copy(file, conn); err != nil {
		logger.Error("Failed to create file '%s': "+err.Error(), iden.ID)
		os.Remove(cfg.App.DataDir + "/" + iden.ID)
	}
}
