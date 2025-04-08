#!/bin/bash

# Create test data directory
mkdir -p ../test_data

# Create temporary directory
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

# Create Cargo project
mkdir -p cert_gen/src
cd cert_gen

# Create Cargo.toml
cat > Cargo.toml << EOF
[package]
name = "cert_gen"
version = "0.1.0"
edition = "2021"

[dependencies]
rcgen = "0.11"
EOF

# Create main.rs
cat > src/main.rs << EOF
use rcgen::{
    Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose,
    IsCa, KeyUsagePurpose, SanType,
};
use std::fs;

fn main() {
    // Generate CA certificate
    let mut ca_params = CertificateParams::default();
    ca_params.distinguished_name.push(DnType::CommonName, "ELASTIC Test CA");
    ca_params.key_usages = vec![
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::CRLSign,
    ];
    ca_params.is_ca = IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    
    let ca_cert = Certificate::from_params(ca_params).unwrap();
    fs::write("../../elastic/crates/elastic-tls/test_data/ca.crt", ca_cert.serialize_pem().unwrap()).unwrap();
    fs::write("../../elastic/crates/elastic-tls/test_data/ca.key", ca_cert.serialize_private_key_pem()).unwrap();

    // Generate server certificate
    let mut server_params = CertificateParams::default();
    server_params.distinguished_name.push(DnType::CommonName, "localhost");
    server_params.subject_alt_names = vec![SanType::DnsName("localhost".to_string())];
    server_params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyEncipherment,
    ];
    server_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    server_params.is_ca = IsCa::NoCa;
    
    let server_cert = Certificate::from_params(server_params).unwrap();
    fs::write("../../elastic/crates/elastic-tls/test_data/server.crt", server_cert.serialize_pem().unwrap()).unwrap();
    fs::write("../../elastic/crates/elastic-tls/test_data/server.key", server_cert.serialize_private_key_pem()).unwrap();
}
EOF

# Build and run
cargo run --release

# Cleanup
cd ../..
rm -rf "$TEMP_DIR"

echo "Test certificates generated successfully" 