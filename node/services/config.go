package services

import (
	"encoding/json"
	"fmt"
	"log"
	"os"
	"strconv"
	"strings"

	"github.com/go-playground/validator/v10"
)

type ConfigFileSSL struct {
	Enable   bool   `json:"enable"`
	CertPath string `json:"cert_path"`
	KeyPath  string `json:"key_path"`
}

type ConfigFile struct {
	SSL  ConfigFileSSL `json:"ssl"`
	Port uint16        `json:"port"`
}

type Config struct {
	Port        uint16
	UseSSL      bool
	SSLCertPath string
	SSLKeyPath  string
}

var configInstance *Config

func validatePort(p string) (uint16, error) {
	port, err := strconv.Atoi(p)

	if err != nil || port > 65534 || port <= 0 {
		return 0, fmt.Errorf("port must be a valid 16 bit unsigned integer, %v was provided", port)
	}

	return uint16(port), nil
}

func validateSSL(key string, cert string) (k string, c string, e error) {
	if key == "" || cert == "" {
		return "", "", fmt.Errorf("if ssl is enabled, a key and a cert must be provided")
	}

	return key, cert, nil
}

func setupFromEnv() error {
	configInstance = &Config{}

	if p := os.Getenv("PORT"); p == "" {
		configInstance.Port = 8080
	} else {
		port, err := validatePort(p)

		if err != nil {
			return err
		}

		configInstance.Port = port
	}

	if useSSL := os.Getenv("SSL_ENABLED"); useSSL == "true" {
		key, cert, err := validateSSL(os.Getenv("SSL_KEY"), os.Getenv("SSL_CERT"))

		if err != nil {
			return err
		}

		configInstance.UseSSL = true
		configInstance.SSLKeyPath = key
		configInstance.SSLCertPath = cert
	} else {
		configInstance.UseSSL = false
	}

	return nil
}

func setupFromFile(file string) error {
	data, err := os.ReadFile(file)

	validate := validator.New()

	if err != nil {
		return err
	}

	config := ConfigFile{}

	if err = json.Unmarshal(data, &config); err != nil {
		return err
	}

	if err = validate.Struct(config); err != nil {
		return err
	}

	configInstance = &Config{
		Port:        config.Port,
		UseSSL:      config.SSL.Enable,
		SSLCertPath: config.SSL.CertPath,
		SSLKeyPath:  config.SSL.KeyPath,
	}

	return nil
}

func GetConfig() Config {
	if configInstance != nil {
		return *configInstance
	} else {
		for _, arg := range os.Args {
			if strings.HasPrefix(arg, "--config=") {
				if argsp := strings.Split(arg, "="); len(argsp) >= 2 {
					method := argsp[1]

					if method == "env" {
						if err := setupFromEnv(); err != nil {
							log.Fatalln(err)
						}
						return *configInstance
					} else {
						if err := setupFromFile(method); err != nil {
							log.Fatalln(err)
						}
					}
				}
				break
			}
		}
	}

	log.Fatalln("--config=<opt> arg must be provided and the options are: env, the path of the config file")
	panic("")
}
