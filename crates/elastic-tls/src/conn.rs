use std::collections::HashMap;
use tokio::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::rustls::{ServerConfig, ClientConfig};
use tokio_rustls::{TlsAcceptor, TlsConnector};
use std::sync::Arc;
use std::io;

pub enum TlsStream {
    Server(tokio_rustls::server::TlsStream<TcpStream>),
    Client(tokio_rustls::client::TlsStream<TcpStream>),
}

pub struct TlsConnection {
    stream: TlsStream,
}

impl TlsConnection {
    pub fn new_server(stream: tokio_rustls::server::TlsStream<TcpStream>) -> Self {
        Self { stream: TlsStream::Server(stream) }
    }

    pub fn new_client(stream: tokio_rustls::client::TlsStream<TcpStream>) -> Self {
        Self { stream: TlsStream::Client(stream) }
    }

    pub async fn read(&mut self, max_size: usize) -> io::Result<Vec<u8>> {
        use tokio::io::AsyncReadExt;
        let mut buffer = vec![0; max_size];
        let n = match &mut self.stream {
            TlsStream::Server(s) => s.read(&mut buffer).await?,
            TlsStream::Client(s) => s.read(&mut buffer).await?,
        };
        buffer.truncate(n);
        Ok(buffer)
    }

    pub async fn write(&mut self, data: &[u8]) -> io::Result<()> {
        use tokio::io::AsyncWriteExt;
        match &mut self.stream {
            TlsStream::Server(s) => s.write_all(data).await,
            TlsStream::Client(s) => s.write_all(data).await,
        }
    }

    pub fn get_peer_certificate(&self) -> Option<Vec<u8>> {
        match &self.stream {
            TlsStream::Server(s) => s.get_ref().1.peer_certificates()
                .map(|certs| certs[0].0.clone()),
            TlsStream::Client(s) => s.get_ref().1.peer_certificates()
                .map(|certs| certs[0].0.clone()),
        }
    }

    pub fn get_protocol_version(&self) -> Option<rustls::ProtocolVersion> {
        match &self.stream {
            TlsStream::Server(s) => s.get_ref().1.protocol_version(),
            TlsStream::Client(s) => s.get_ref().1.protocol_version(),
        }
    }

    pub fn get_cipher_suite(&self) -> Option<rustls::SupportedCipherSuite> {
        match &self.stream {
            TlsStream::Server(s) => s.get_ref().1.negotiated_cipher_suite(),
            TlsStream::Client(s) => s.get_ref().1.negotiated_cipher_suite(),
        }
    }
}

#[derive(Clone)]
pub struct ConnectionManager {
    pub(crate) listener: Arc<Mutex<Option<TcpListener>>>,
    connections: Arc<Mutex<HashMap<u32, TlsConnection>>>,
    next_handle: Arc<Mutex<u32>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            listener: Arc::new(Mutex::new(None)),
            connections: Arc::new(Mutex::new(HashMap::new())),
            next_handle: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn set_listener(&self, listener: TcpListener) {
        let mut guard = self.listener.lock().await;
        *guard = Some(listener);
    }

    pub async fn accept(&self, config: Arc<ServerConfig>) -> Result<u32, rustls::Error> {
        let listener = self.listener.lock().await;
        let listener = listener.as_ref()
            .ok_or_else(|| rustls::Error::General("No listener bound".to_string()))?;

        let (stream, _) = listener.accept().await
            .map_err(|e| rustls::Error::General(format!("Accept failed: {}", e)))?;

        let acceptor = TlsAcceptor::from(config);
        let stream = acceptor.accept(stream).await
            .map_err(|e| rustls::Error::General(format!("TLS accept failed: {}", e)))?;

        let mut handle = self.next_handle.lock().await;
        let connection_handle = *handle;
        *handle += 1;

        let mut connections = self.connections.lock().await;
        connections.insert(connection_handle, TlsConnection::new_server(stream));

        Ok(connection_handle)
    }

    pub async fn connect(&self, config: Arc<ClientConfig>, hostname: &str, port: u16) -> Result<u32, rustls::Error> {
        let stream = TcpStream::connect((hostname, port)).await
            .map_err(|e| rustls::Error::General(format!("Connect failed: {}", e)))?;

        let connector = TlsConnector::from(config);
        let domain = rustls::ServerName::try_from(hostname)
            .map_err(|_| rustls::Error::General("Invalid hostname".to_string()))?;

        let stream = connector.connect(domain, stream).await
            .map_err(|e| rustls::Error::General(format!("TLS connect failed: {}", e)))?;

        let mut handle = self.next_handle.lock().await;
        let connection_handle = *handle;
        *handle += 1;

        let mut connections = self.connections.lock().await;
        connections.insert(connection_handle, TlsConnection::new_client(stream));

        Ok(connection_handle)
    }

    pub async fn close(&self, handle: u32) -> Result<(), rustls::Error> {
        let mut connections = self.connections.lock().await;
        connections.remove(&handle)
            .ok_or_else(|| rustls::Error::General("Connection not found".to_string()))?;
        Ok(())
    }

    pub async fn write(&self, handle: u32, data: &[u8]) -> Result<(), rustls::Error> {
        let mut connections = self.connections.lock().await;
        let connection = connections.get_mut(&handle)
            .ok_or_else(|| rustls::Error::General("Connection not found".to_string()))?;

        connection.write(data).await
            .map_err(|e| rustls::Error::General(format!("Write failed: {}", e)))
    }

    pub async fn read(&self, handle: u32, max_size: usize) -> Result<Vec<u8>, rustls::Error> {
        let mut connections = self.connections.lock().await;
        let connection = connections.get_mut(&handle)
            .ok_or_else(|| rustls::Error::General("Connection not found".to_string()))?;

        connection.read(max_size).await
            .map_err(|e| rustls::Error::General(format!("Read failed: {}", e)))
    }

    pub async fn get_peer_certificate(&self, handle: u32) -> Result<Option<Vec<u8>>, rustls::Error> {
        let connections = self.connections.lock().await;
        let connection = connections.get(&handle)
            .ok_or_else(|| rustls::Error::General("Connection not found".to_string()))?;

        Ok(connection.get_peer_certificate())
    }

    pub async fn get_protocol_version(&self, handle: u32) -> Result<Option<rustls::ProtocolVersion>, rustls::Error> {
        let connections = self.connections.lock().await;
        let connection = connections.get(&handle)
            .ok_or_else(|| rustls::Error::General("Connection not found".to_string()))?;

        Ok(connection.get_protocol_version())
    }

    pub async fn get_cipher_suite(&self, handle: u32) -> Result<Option<rustls::SupportedCipherSuite>, rustls::Error> {
        let connections = self.connections.lock().await;
        let connection = connections.get(&handle)
            .ok_or_else(|| rustls::Error::General("Connection not found".to_string()))?;

        Ok(connection.get_cipher_suite())
    }
}