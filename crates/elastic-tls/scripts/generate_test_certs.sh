#!/bin/bash

# Create testdata directory
mkdir -p ../testdata

# Generate private key
openssl genrsa -out ../testdata/server.key 2048

# Generate certificate signing request
openssl req -new -key ../testdata/server.key -out ../testdata/server.csr -subj "/CN=localhost"

# Generate self-signed certificate
openssl x509 -req -days 365 -in ../testdata/server.csr -signkey ../testdata/server.key -out ../testdata/server.crt

# Clean up CSR
rm ../testdata/server.csr

echo "Test certificates generated successfully" 