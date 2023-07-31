package transport

import (
	"bytes"
	"encoding/gob"

	"github.com/valyala/bytebufferpool"
)

type RequestType uint8

const (
	RequestTypeRead  = 1
	RequestTypeWrite = 2
)

type IdenPayload struct {
	ID     string
	Random string
	Token  string
	Type   RequestType
}

func (p *IdenPayload) Encode() ([]byte, error) {
	buffer := bytebufferpool.Get()

	enc := gob.NewEncoder(buffer)

	if err := enc.Encode(p); err != nil {
		return nil, err
	}

	return buffer.Bytes(), nil
}

func DecodeIdenPayload(b []byte, p *IdenPayload) error {
	buffer := bytes.NewReader(b)
	dec := gob.NewDecoder(buffer)

	if err := dec.Decode(p); err != nil {
		return err
	}

	return nil
}
