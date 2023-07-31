package tcp

import (
	"crypto/rand"
	"crypto/tls"
	"net"

	"github.com/go-playground/validator/v10"
	"github.com/zanz1n/downloader/shared/logger"
	"github.com/zanz1n/downloader/shared/transport"
)

var (
	serverLogger = logger.NewLogger("tcp_log")
	validate     = validator.New()
)

func NewServer() *Server {
	return &Server{}
}

type Server struct{}

func (s *Server) acceptLoop(ln net.Listener) {
	for {
		conn, err := ln.Accept()
		if err != nil {
			logger.Debug(err.Error())
			break
		}

		go s.Handle(conn)
	}
}

func (s *Server) Handle(conn net.Conn) {
	serverLogger.Info("Incomming conn " + conn.RemoteAddr().String())
	var err error
	defer conn.Close()

	buf := make([]byte, 1024)

	if _, err = conn.Read(buf); err != nil {
		serverLogger.Info("Conn %s closed too soon",
			conn.RemoteAddr().String())
		return
	}

	iden := transport.IdenPayload{}
	if err = transport.DecodeIdenPayload(buf, &iden); err != nil {
		serverLogger.Info("Conn %s send a poorly encoded iden payload: "+
			err.Error(), conn.RemoteAddr().String())
		return
	}
	if err = validate.Struct(&iden); err != nil {
		serverLogger.Info("Conn %s send an invalid iden payload: " + err.Error())
	}

	if err = validateIden(&iden); err != nil {
		serverLogger.Info("Conn %s send an invalid signature", conn.RemoteAddr().String())
		return
	}

	switch iden.Type {
	case transport.RequestTypeRead:
		s.HandleRead(conn, &iden)
	case transport.RequestTypeWrite:
		s.HandleWrite(conn, &iden)
	default:
		serverLogger.Info("Conn %s send an invalid iden payload",
			conn.RemoteAddr().String())
		return
	}

	serverLogger.Info("Conn " + conn.RemoteAddr().String() + " closed")
}

func (s *Server) ListenAndServeTLS(addr, certFile, keyFile string) error {
	cert, err := tls.LoadX509KeyPair(certFile, keyFile)
	if err != nil {
		return err
	}

	ln, err := tls.Listen("tcp", addr, &tls.Config{
		Rand:               rand.Reader,
		Certificates:       []tls.Certificate{cert},
		InsecureSkipVerify: true,
	})
	if err != nil {
		return err
	}

	serverLogger.Info("Listening with tls for " + addr)
	s.acceptLoop(ln)

	return nil
}

func (s *Server) ListenAndServe(addr string) error {
	ln, err := net.Listen("tcp", addr)
	if err != nil {
		return err
	}

	serverLogger.Info("Listening for " + addr)
	s.acceptLoop(ln)

	return nil
}

func (s *Server) MustListenAndServe(addr string) {
	if err := s.ListenAndServe(addr); err != nil {
		serverLogger.Fatal(err)
	}
}

func (s *Server) MustListenAndServeTLS(addr, certFile, keyFile string) {
	if err := s.ListenAndServeTLS(addr, certFile, keyFile); err != nil {
		serverLogger.Fatal(err)
	}
}
