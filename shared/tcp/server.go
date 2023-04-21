package tcp

import (
	"fmt"
	"log"
	"net"
	"os"
	"os/signal"
	"sync"
	"syscall"
)

type TcpServer struct {
	conns   map[uint64]*net.Conn
	connCtr uint64
	NetLn   net.Listener
	handler func(conn net.Conn) error
	ConnsM  sync.Mutex
	endCh   chan os.Signal
}

func NewServer() *TcpServer {
	return &TcpServer{
		conns:   make(map[uint64]*net.Conn),
		connCtr: 0,
		handler: func(conn net.Conn) error {
			return nil
		},
		endCh:  make(chan os.Signal),
		ConnsM: sync.Mutex{},
	}
}

func (s *TcpServer) GetConn(id uint64) net.Conn {
	s.ConnsM.Lock()
	defer s.ConnsM.Unlock()
	if conn, ok := s.conns[id]; ok {
		return *conn
	} else {
		return nil
	}
}

func (s *TcpServer) GetConns() []net.Conn {
	conns := make([]net.Conn, len(s.conns))
	i := 0

	s.ConnsM.Lock()
	defer s.ConnsM.Unlock()

	for _, conn := range s.conns {
		conns[i] = *conn
		i++
	}
	return conns
}

func (s *TcpServer) Handle(handleFunc func(conn net.Conn) error) {
	s.handler = handleFunc
}

func (s *TcpServer) addConn(conn net.Conn) uint64 {
	s.ConnsM.Lock()
	defer s.ConnsM.Unlock()

	s.connCtr++
	i := s.connCtr
	s.conns[i] = &conn

	return i
}

func (s *TcpServer) removeConn(id uint64) {
	s.ConnsM.Lock()
	defer s.ConnsM.Unlock()

	delete(s.conns, id)
}

func (s *TcpServer) connHandler(conn net.Conn) {
	i := s.addConn(conn)
	defer s.removeConn(i)
	s.handler(conn)
}

func (s *TcpServer) acceptLoop() {
	for {
		conn, err := s.NetLn.Accept()
		if err != nil {
			break
		}

		go s.connHandler(conn)
	}
}

func (s *TcpServer) Listen(port uint16, bind string) {
	var err error
	s.NetLn, err = net.Listen("tcp", fmt.Sprintf("%s:%v", bind, port))

	if err != nil {
		return
	}

	log.Printf("Listening on port %v", s.NetLn.Addr().String())

	signal.Notify(s.endCh, syscall.SIGINT, syscall.SIGTERM, os.Interrupt)

	go s.acceptLoop()

	<-s.endCh

	s.Close()
}

func (s *TcpServer) Close() {
	log.Println("Stopping ...")

	s.NetLn.Close()
}
