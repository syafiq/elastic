use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{
    rustls::{ClientConfig, ServerConfig},
    TlsAcceptor, TlsConnector,
    server::TlsStream as ServerTlsStream,
    client::TlsStream as ClientTlsStream,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug)]
pub enum ConnError {
    BindFailed(String),
    AcceptFailed(String),
    ConnectFailed(String),
    InvalidHandle(String),
    WriteFailed(String),
    ReadFailed(String),
    CloseFailed(String),
    InvalidCertificate(String),
}

impl std::fmt::Display for ConnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnError::BindFailed(e) => write!(f, "Bind failed: {}", e),
            ConnError::AcceptFailed(e) => write!(f, "Accept failed: {}", e),
            ConnError::ConnectFailed(e) => write!(f, "Connect failed: {}", e),
            ConnError::InvalidHandle(e) => write!(f, "Invalid handle: {}", e),
            ConnError::WriteFailed(e) => write!(f, "Write failed: {}", e),
            ConnError::ReadFailed(e) => write!(f, "Read failed: {}", e),
            ConnError::CloseFailed(e) => write!(f, "Close failed: {}", e),
            ConnError::InvalidCertificate(e) => write!(f, "Invalid certificate: {}", e),
        }
    }
}

impl std::error::Error for ConnError {}

#[derive(Clone)]
pub struct ConnectionManager {
    listener: Arc<Mutex<Option<TcpListener>>>,
    connections: Arc<Mutex<Vec<TlsStreamType>>>,
}

enum TlsStreamType {
    Server(Arc<Mutex<ServerTlsStream<TcpStream>>>),
    Client(Arc<Mutex<ClientTlsStream<TcpStream>>>),
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            listener: Arc::new(Mutex::new(None)),
            connections: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn bind(&self, addr: &str) -> Result<(), ConnError> {
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| ConnError::BindFailed(e.to_string()))?;
        *self.listener.lock().await = Some(listener);
        Ok(())
    }

    pub async fn accept(&self, server_config: Arc<ServerConfig>) -> Result<u32, ConnError> {
        let listener = self.listener.lock().await;
        let listener = listener.as_ref()
            .ok_or_else(|| ConnError::AcceptFailed("No listener bound".to_string()))?;
        
        let (stream, _) = listener.accept()
            .await
            .map_err(|e| ConnError::AcceptFailed(e.to_string()))?;
        
        let acceptor = TlsAcceptor::from(server_config);
        let tls_stream = acceptor.accept(stream)
            .await
            .map_err(|e| ConnError::AcceptFailed(e.to_string()))?;
        
        let mut connections = self.connections.lock().await;
        let handle = connections.len() as u32;
        connections.push(TlsStreamType::Server(Arc::new(Mutex::new(tls_stream))));
        Ok(handle)
    }

    pub async fn connect(&self, addr: &str, client_config: Arc<ClientConfig>) -> Result<u32, ConnError> {
        let stream = TcpStream::connect(addr)
            .await
            .map_err(|e| ConnError::ConnectFailed(e.to_string()))?;
        
        let connector = TlsConnector::from(client_config);
        let tls_stream = connector.connect(addr.try_into().unwrap(), stream)
            .await
            .map_err(|e| ConnError::ConnectFailed(e.to_string()))?;
        
        let mut connections = self.connections.lock().await;
        let handle = connections.len() as u32;
        connections.push(TlsStreamType::Client(Arc::new(Mutex::new(tls_stream))));
        Ok(handle)
    }

    pub async fn close(&self, handle: u32) -> Result<(), ConnError> {
        let mut connections = self.connections.lock().await;
        if handle as usize >= connections.len() {
            return Err(ConnError::InvalidHandle(format!("Invalid handle: {}", handle)));
        }
        connections.remove(handle as usize);
        Ok(())
    }

    pub async fn write(&self, handle: u32, data: &[u8]) -> Result<(), ConnError> {
        let connections = self.connections.lock().await;
        let stream = connections.get(handle as usize)
            .ok_or_else(|| ConnError::InvalidHandle(format!("Invalid handle: {}", handle)))?;
        
        match stream {
            TlsStreamType::Server(stream) => {
                let mut stream = stream.lock().await;
                stream.write_all(data).await
                    .map_err(|e| ConnError::WriteFailed(e.to_string()))?;
            }
            TlsStreamType::Client(stream) => {
                let mut stream = stream.lock().await;
                stream.write_all(data).await
                    .map_err(|e| ConnError::WriteFailed(e.to_string()))?;
            }
        }
        Ok(())
    }

    pub async fn read(&self, handle: u32, buf: &mut [u8]) -> Result<usize, ConnError> {
        let connections = self.connections.lock().await;
        let stream = connections.get(handle as usize)
            .ok_or_else(|| ConnError::InvalidHandle(format!("Invalid handle: {}", handle)))?;
        
        match stream {
            TlsStreamType::Server(stream) => {
                let mut stream = stream.lock().await;
                stream.read(buf).await
                    .map_err(|e| ConnError::ReadFailed(e.to_string()))
            }
            TlsStreamType::Client(stream) => {
                let mut stream = stream.lock().await;
                stream.read(buf).await
                    .map_err(|e| ConnError::ReadFailed(e.to_string()))
            }
        }
    }

    pub async fn get_cipher_suite(&self, handle: u32) -> Result<String, ConnError> {
        let connections = self.connections.lock().await;
        let stream = connections.get(handle as usize)
            .ok_or_else(|| ConnError::InvalidHandle(format!("Invalid handle: {}", handle)))?;
        
        match stream {
            TlsStreamType::Server(stream) => {
                let stream = stream.lock().await;
                Ok(format!("{:?}", stream.get_ref().1.negotiated_cipher_suite().unwrap()))
            }
            TlsStreamType::Client(stream) => {
                let stream = stream.lock().await;
                Ok(format!("{:?}", stream.get_ref().1.negotiated_cipher_suite().unwrap()))
            }
        }
    }

    pub async fn get_protocol_version(&self, handle: u32) -> Result<String, ConnError> {
        let connections = self.connections.lock().await;
        let stream = connections.get(handle as usize)
            .ok_or_else(|| ConnError::InvalidHandle(format!("Invalid handle: {}", handle)))?;
        
        match stream {
            TlsStreamType::Server(stream) => {
                let stream = stream.lock().await;
                Ok(format!("{:?}", stream.get_ref().1.protocol_version().unwrap()))
            }
            TlsStreamType::Client(stream) => {
                let stream = stream.lock().await;
                Ok(format!("{:?}", stream.get_ref().1.protocol_version().unwrap()))
            }
        }
    }

    pub async fn get_peer_certificate(&self, handle: u32) -> Result<Vec<u8>, ConnError> {
        let connections = self.connections.lock().await;
        let stream = connections.get(handle as usize)
            .ok_or_else(|| ConnError::InvalidHandle(format!("Invalid handle: {}", handle)))?;
        
        match stream {
            TlsStreamType::Server(stream) => {
                let stream = stream.lock().await;
                let certs = stream.get_ref().1.peer_certificates()
                    .ok_or_else(|| ConnError::InvalidCertificate("No peer certificates".to_string()))?;
                Ok(certs[0].as_ref().to_vec())
            }
            TlsStreamType::Client(stream) => {
                let stream = stream.lock().await;
                let certs = stream.get_ref().1.peer_certificates()
                    .ok_or_else(|| ConnError::InvalidCertificate("No peer certificates".to_string()))?;
                Ok(certs[0].as_ref().to_vec())
            }
        }
    }
}