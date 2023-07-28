package config

import (
	"errors"
	"os"
	"strconv"
)

func GetConfig() *Config {
	return config
}

type Config struct {
	Key            string
	PostgresURI    string
	Port           int
	EnableTLS      bool
	TLSCertPath    string
	TLSKeyPath     string
	JWTHmacKey     string
	JWTPrivkeyPath string
	JWTPubkeyPath  string
}

var config *Config

func (c *Config) GetKey() string {
	return c.Key
}

func (c *Config) SetKey(v string) {
	c.Key = v
}

func FromEnv() error {
	config = &Config{
		Key:            os.Getenv("APP_KEY"),
		Port:           8080,
		EnableTLS:      false,
		PostgresURI:    os.Getenv("DATABASE_URI"),
		TLSCertPath:    "",
		TLSKeyPath:     "",
		JWTHmacKey:     os.Getenv("APP_JWT_HMAC_KEY"),
		JWTPrivkeyPath: os.Getenv("APP_JWT_ED_PRIVKEY"),
		JWTPubkeyPath:  os.Getenv("APP_JWT_ED_PUBKEY"),
	}

	if port := os.Getenv("APP_PORT"); port != "" {
		i, err := strconv.Atoi(port)
		if err != nil {
			return errors.New("config: 'APP_PORT' must be a valid integer")
		}
		config.Port = i
	}

	if tls := os.Getenv("APP_ENABLE_TLS"); tls == "true" || tls == "1" {
		tlsCertPath, tlsKeyPath := os.Getenv("APP_SSL_CERT"), os.Getenv("APP_SSL_KEY")

		if tlsCertPath == "" || tlsKeyPath == "" {
			return errors.New("config: if 'APP_ENABLE_TLS' is set to true, " +
				"'APP_SSL_KEY' and 'APP_SSL_CERT' paths must be provided")
		}

		config.EnableTLS = true
		config.TLSCertPath = tlsCertPath
		config.TLSKeyPath = tlsKeyPath
	}

	switch {
	case len(config.Key) < 2:
		return errors.New("config: 'APP_KEY' must be provided")
	case len(config.JWTHmacKey) < 2:
		return errors.New("config: 'APP_JWT_HMAC_KEY' must be provided")
	case config.PostgresURI == "":
		return errors.New("config: 'DATABASE_URI' must be provided")
	}

	return nil
}
