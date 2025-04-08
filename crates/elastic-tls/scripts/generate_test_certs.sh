#!/bin/bash

# Create test data directory
mkdir -p test_data

# Generate CA private key and certificate
openssl req -x509 -newkey rsa:4096 -days 365 -nodes \
    -keyout test_data/ca.key -out test_data/ca.crt \
    -subj "/CN=ELASTIC Test CA"

# Generate server private key
openssl genrsa -out test_data/server.key 4096

# Generate server CSR
openssl req -new -key test_data/server.key \
    -out test_data/server.csr \
    -subj "/CN=localhost"

# Generate server certificate
openssl x509 -req -in test_data/server.csr \
    -CA test_data/ca.crt -CAkey test_data/ca.key -CAcreateserial \
    -out test_data/server.crt -days 365 \
    -extfile <(printf "subjectAltName=DNS:localhost\nkeyUsage=digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth")

echo "Test certificates generated successfully" 