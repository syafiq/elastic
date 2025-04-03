#!/bin/bash

# Create fixtures directory
mkdir -p tests/fixtures

# Generate CA certificate and key
openssl req -x509 -newkey rsa:4096 -keyout tests/fixtures/ca.key -out tests/fixtures/ca.crt -days 365 -nodes -subj "/CN=Test CA"

# Generate server certificate and key
openssl req -newkey rsa:4096 -keyout tests/fixtures/server.key -out tests/fixtures/server.csr -nodes -subj "/CN=localhost"
openssl x509 -req -in tests/fixtures/server.csr -CA tests/fixtures/ca.crt -CAkey tests/fixtures/ca.key -CAcreateserial -out tests/fixtures/server.crt -days 365

# Clean up CSR file
rm tests/fixtures/server.csr 