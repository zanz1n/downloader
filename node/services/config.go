package services

import (
	"encoding/json"
	"fmt"
	"log"
	"os"
	"strconv"

	"github.com/go-playground/validator/v10"
)

type ConfigFileSSL struct {
	Enable   bool   `json:"enable"`
	CertPath string `json:"cert_path"`
	KeyPath  string `json:"key_path"`
}

type ConfigFile struct {
	Port       uint16        `json:"port" validate:"required"`
	ManagerKey string        `json:"manager_key"`
	DataPath   string        `json:"data_path" validate:"required"`
	JwtKey     string        `json:"jwt_key" validate:"required"`
	SSL        ConfigFileSSL `json:"ssl" validate:"required"`
}

type Config struct {
	Port        uint16
	ManagerKey  string
	DataPath    string
	JwtKey      string
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

	if managerKey := os.Getenv("MANAGER_KEY"); managerKey == "" {
		return fmt.Errorf("manager key is required")
	} else {
		configInstance.ManagerKey = managerKey
	}

	if jwtKey := os.Getenv("JWT_KEY"); jwtKey == "" {
		return fmt.Errorf("jwt key is required")
	} else {
		configInstance.JwtKey = jwtKey
	}

	if dataPath := os.Getenv("DATA_PATH"); dataPath == "" {
		return fmt.Errorf("jwt key is required")
	} else {
		configInstance.DataPath = dataPath
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

	key, cert, err := validateSSL(config.SSL.KeyPath, config.SSL.CertPath)

	if err != nil {
		return err
	}

	if len(config.ManagerKey) < 1 {
		return fmt.Errorf("manager key is empty")
	}

	configInstance = &Config{
		Port:        config.Port,
		UseSSL:      config.SSL.Enable,
		SSLCertPath: cert,
		SSLKeyPath:  key,
		ManagerKey:  config.ManagerKey,
		DataPath:    config.DataPath,
	}

	return nil
}

func GetConfig() Config {
	if configInstance != nil {
		return *configInstance
	} else {
		for i, arg := range os.Args {
			if arg == "--config" {
				if len(os.Args) > i {
					method := os.Args[i+1]
					if method == "env" {
						if err := setupFromEnv(); err != nil {
							log.Fatalln(err)
						}
						return *configInstance
					} else {
						if err := setupFromFile(method); err != nil {
							log.Fatalln(err)
						}
						return *configInstance
					}
				}
				break
			}
		}
	}

	log.Fatalln("--config <opt> arg must be provided and the options are: env, the path of the config file")
	panic("")
}
