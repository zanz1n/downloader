package server

type DataBody struct {
	Data    any `json:"data"`
	Message any `json:"message"`
}

func dataBody(v any, msg string) *DataBody {
	return &DataBody{
		Data:    v,
		Message: msg,
	}
}
