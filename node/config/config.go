package config

import (
	"errors"

	"github.com/go-playground/validator/v10"
	"github.com/google/uuid"
)

type Config struct {
	InstanceID string     `json:"id" yaml:"id"`
	Key        string     `json:"key" yaml:"key"`
	App        *ConfigApp `json:"app" yaml:"app"`
	TCP        *ConfigTcp `json:"tcp" yaml:"tcp"`
}

type ConfigApp struct {
	DataDir string     `json:"dataDir" yaml:"data-dir" default:"/var/downloader/data"`
	Port    int        `json:"port" yaml:"port" default:"8080"`
	SSL     *ConfigSSl `json:"ssl" yaml:"ssl"`
}

type ConfigSSl struct {
	Enabled         bool   `json:"enabled" yaml:"enabled"`
	CertificateFile string `json:"cert" yaml:"cert"`
	KeyFile         string `json:"key" yaml:"key"`
}

type ConfigTcp struct {
	Enabled bool       `json:"enabled" yaml:"enabled"`
	Port    int        `json:"port" yaml:"port" default:"2022"`
	SSL     *ConfigSSl `json:"ssl" yaml:"ssl"`
}

var validate = validator.New()

func (c *Config) IsValid() error {
	var err error

	if err = validate.Struct(c); err != nil {
		return errors.New("config: " + err.Error())
	}
	if _, err = uuid.Parse(c.InstanceID); err != nil {
		return errors.New("config: 'id' prop must be a valid uuid string")
	}

	switch {
	case c.App == nil:
		return errors.New("config: 'app' prop must be provided")
	case c.TCP == nil:
		return errors.New("config: 'tcp' prop must be provided")
	}

	if err = validateSsl(c.App.SSL); err != nil {
		return err
	}
	if err = validateSsl(c.TCP.SSL); err != nil {
		return err
	}

	return nil
}

func validateSsl(ssl *ConfigSSl) error {
	if ssl == nil {
		return errors.New("config: 'ssl' prop must not be null")
	}

	if !ssl.Enabled {
		return nil
	}

	if ssl.KeyFile == "" || ssl.CertificateFile == "" {
		return errors.New("config: if ssl is enabled the 'key' and 'cert' props must be provided")
	}

	return nil
}
