package config

import (
	"errors"
	"io"
	"os"
	"sync"

	"github.com/go-playground/validator/v10"
	"github.com/google/uuid"
	"github.com/zanz1n/downloader/shared/logger"
	"gopkg.in/yaml.v3"
)

func GetConfig() *Config {
	return config
}

type Config struct {
	// Is the id of the running node registered in the datbase
	InstanceID string `json:"id" yaml:"id"`
	// Can and will be change during runtime. Is the Key that will sign
	// every restricted api request.
	Key string     `json:"key" yaml:"key"`
	App *ConfigApp `json:"app" yaml:"app"`
	// This configures the tcp server that will run for streaming files in
	// a better speed
	TCP *ConfigTcp `json:"tcp" yaml:"tcp"`
}

type ConfigApp struct {
	// The directory that the program will try to save the files
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

var (
	validate = validator.New()
	cfgPath  string
	config   *Config
	fileMu   = sync.RWMutex{}
)

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

	if err = validateSSL(c.App.SSL); err != nil {
		return err
	}
	if c.TCP.Enabled {
		if err = validateSSL(c.TCP.SSL); err != nil {
			return err
		}
	}

	return nil
}

func (c *Config) GetKey() string {
	fileMu.RLock()
	defer fileMu.RUnlock()

	return c.Key
}

func (c *Config) SetKey(v string) {
	c.Key = v
	dumpToFile()
}

func validateSSL(ssl *ConfigSSl) error {
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

func dumpToFile() error {
	fileMu.Lock()
	defer fileMu.Unlock()

	file, err := os.OpenFile(cfgPath, os.O_RDWR|os.O_CREATE|os.O_TRUNC, 0755)

	if err != nil {
		return errors.New("config: failed to open config file")
	}
	defer file.Close()

	buf, err := yaml.Marshal(config)

	if err != nil {
		return errors.New("config: failed to marshal configuration: " + err.Error())
	}

	if err := file.Truncate(0); err != nil {
		return errors.New("config: failed to truncate config file: " + err.Error())
	}

	if _, err = file.Write(buf); err != nil {
		return errors.New("config: failed to write config file: " + err.Error())
	}

	return nil
}

func DumpToFile() {
	if err := dumpToFile(); err != nil {
		logger.Error(err.Error())
	}
}

func FromYamlFile(path string) error {
	cfgPath = path
	file, err := os.Open(path)

	if err != nil {
		return errors.New("config: could not open file " + path)
	}

	buf, err := io.ReadAll(file)

	if err != nil {
		return errors.New("config: failed to read config file")
	}

	config = &Config{}

	yaml.Unmarshal(buf, config)

	return config.IsValid()
}

func MustFromYamlFile(path string) {
	if err := FromYamlFile(path); err != nil {
		logger.Fatal(err)
	}
}
