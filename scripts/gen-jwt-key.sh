#!/bin/bash

openssl genpkey -algorithm ed25519 -out ./data/certs/jwt-key.pem
openssl pkey -in ./data/certs/jwt-key.pem -pubout -out ./data/certs/jwt-cert.pem
