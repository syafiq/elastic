use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::{
    client::TlsStream as ClientTlsStream,
    server::TlsStream as ServerTlsStream,
    TlsConnector, TlsAcceptor,
};
use rustls::{self, ClientConfig, ServerConfig};

pub enum TlsConnection {
    Client(Box<ClientTlsStream<TcpStream>>),
    Server(Box<ServerTlsStream<TcpStream>>),
}

#[derive(Clone)]
pub struct ConnectionManager {
    connections: Arc<Mutex<HashMap<u32, TlsConnection>>>,
    next_handle: Arc<Mutex<u32>>,
    listener: Arc<Mutex<Option<Arc<TcpListener>>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            next_handle: Arc::new(Mutex::new(1)),
            listener: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn connect(
        &self,
        client_config: Arc<ClientConfig>,
        hostname: &str,
        port: u16,
    ) -> Result<u32, rustls::Error> {
        let stream = TcpStream::connect((hostname, port)).await
            .map_err(|e| rustls::Error::General(format!("Failed to connect: {}", e)))?;
        
        let domain = rustls::ServerName::try_from(hostname)
            .map_err(|_| rustls::Error::General("Invalid hostname".to_string()))?;
        
        let tls_stream = TlsConnector::from(client_config)
            .connect(domain, stream)
            .await
            .map_err(|e| rustls::Error::General(format!("TLS connection failed: {}", e)))?;
        
        let handle = {
            let mut next_handle = self.next_handle.lock().unwrap();
            let handle = *next_handle;
            *next_handle += 1;
            handle
        };
        
        self.connections.lock().unwrap().insert(handle, TlsConnection::Client(Box::new(tls_stream)));
        Ok(handle)
    }

    pub async fn bind(&self, port: u16) -> Result<(), rustls::Error> {
        println!("Attempting to bind to port {}", port);
        let listener = TcpListener::bind(("127.0.0.1", port)).await
            .map_err(|e| rustls::Error::General(format!("Failed to bind to port {}: {}", port, e)))?;
        println!("Successfully bound to port {}", port);
        *self.listener.lock().unwrap() = Some(Arc::new(listener));
        Ok(())
    }

    pub async fn accept(
        &self,
        server_config: Arc<ServerConfig>,
    ) -> Result<u32, rustls::Error> {
        let listener = self.listener.lock().unwrap()
            .as_ref()
            .ok_or_else(|| rustls::Error::General("No listener bound".to_string()))?
            .clone();

        println!("Waiting for incoming connection...");
        let (stream, _) = listener.accept().await
            .map_err(|e| rustls::Error::General(format!("Failed to accept connection: {}", e)))?;
        println!("Accepted TCP connection, performing TLS handshake...");
        
        let tls_stream = TlsAcceptor::from(server_config)
            .accept(stream)
            .await
            .map_err(|e| rustls::Error::General(format!("TLS accept failed: {}", e)))?;
        println!("TLS handshake completed successfully");
        
        let handle = {
            let mut next_handle = self.next_handle.lock().unwrap();
            let handle = *next_handle;
            *next_handle += 1;
            handle
        };
        
        self.connections.lock().unwrap().insert(handle, TlsConnection::Server(Box::new(tls_stream)));
        Ok(handle)
    }

    pub fn close(&self, handle: u32) -> Result<(), rustls::Error> {
        let mut connections = self.connections.lock().unwrap();
        if connections.remove(&handle).is_some() {
            Ok(())
        } else {
            Err(rustls::Error::General("Connection not found".to_string()))
        }
    }

    pub async fn write(&self, handle: u32, data: &[u8]) -> Result<(), rustls::Error> {
        let mut connections = self.connections.lock().unwrap();
        let connection = connections.get_mut(&handle)
            .ok_or_else(|| rustls::Error::General("Connection not found".to_string()))?;
        
        match connection {
            TlsConnection::Client(stream) => {
                stream.write_all(data).await
                    .map_err(|e| rustls::Error::General(format!("Write failed: {}", e)))?;
            }
            TlsConnection::Server(stream) => {
                stream.write_all(data).await
                    .map_err(|e| rustls::Error::General(format!("Write failed: {}", e)))?;
            }
        }
        Ok(())
    }

    pub async fn read(&self, handle: u32, max_size: usize) -> Result<Vec<u8>, rustls::Error> {
        let mut connections = self.connections.lock().unwrap();
        let connection = connections.get_mut(&handle)
            .ok_or_else(|| rustls::Error::General("Connection not found".to_string()))?;
        
        let mut buffer = vec![0u8; max_size];
        let n = match connection {
            TlsConnection::Client(stream) => {
                stream.read(&mut buffer).await
                    .map_err(|e| rustls::Error::General(format!("Read failed: {}", e)))?
            }
            TlsConnection::Server(stream) => {
                stream.read(&mut buffer).await
                    .map_err(|e| rustls::Error::General(format!("Read failed: {}", e)))?
            }
        };
        
        buffer.truncate(n);
        Ok(buffer)
    }

    pub fn get_peer_certificate(&self, handle: u32) -> Result<Option<Vec<u8>>, rustls::Error> {
        let connections = self.connections.lock().unwrap();
        let connection = connections.get(&handle)
            .ok_or_else(|| rustls::Error::General("Connection not found".to_string()))?;
        
        match connection {
            TlsConnection::Client(stream) => {
                Ok(stream.get_ref().1.peer_certificates()
                    .and_then(|certs| certs.first())
                    .map(|cert| cert.0.clone()))
            }
            TlsConnection::Server(stream) => {
                Ok(stream.get_ref().1.peer_certificates()
                    .and_then(|certs| certs.first())
                    .map(|cert| cert.0.clone()))
            }
        }
    }

    pub fn get_protocol_version(&self, handle: u32) -> Result<Option<rustls::ProtocolVersion>, rustls::Error> {
        let connections = self.connections.lock().unwrap();
        let connection = connections.get(&handle)
            .ok_or_else(|| rustls::Error::General("Connection not found".to_string()))?;
        
        match connection {
            TlsConnection::Client(stream) => Ok(stream.get_ref().1.protocol_version()),
            TlsConnection::Server(stream) => Ok(stream.get_ref().1.protocol_version()),
        }
    }

    pub fn get_cipher_suite(&self, handle: u32) -> Result<Option<rustls::SupportedCipherSuite>, rustls::Error> {
        let connections = self.connections.lock().unwrap();
        let connection = connections.get(&handle)
            .ok_or_else(|| rustls::Error::General("Connection not found".to_string()))?;
        
        match connection {
            TlsConnection::Client(stream) => Ok(stream.get_ref().1.negotiated_cipher_suite()),
            TlsConnection::Server(stream) => Ok(stream.get_ref().1.negotiated_cipher_suite()),
        }
    }
}