use std::sync::Arc;
use tokio_rustls::rustls::{self, ClientConfig, ServerConfig};
use crate::{TlsError, TlsConfig};

pub struct WasmTlsContext {
    use_sevsnp: bool,
}

impl WasmTlsContext {
    pub fn new() -> Self {
        Self {
            use_sevsnp: Self::is_sevsnp_available(),
        }
    }

    fn is_sevsnp_available() -> bool {
        std::env::var("ELASTIC_SEV_SNP").is_ok()
    }

    pub fn create_client_config(&self, config: &TlsConfig) -> Result<Arc<ClientConfig>, TlsError> {
        let mut client_config = if config.verify_peer {
            let mut root_store = rustls::RootCertStore::empty();
            root_store.add_trust_anchors(
                webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
                    rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                        ta.subject,
                        ta.spki,
                        ta.name_constraints,
                    )
                })
            );

            ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(root_store)
                .with_no_client_auth()
        } else {
            ClientConfig::builder()
                .with_safe_defaults()
                .with_custom_certificate_verifier(Arc::new(crate::AcceptAllVerifier))
                .with_no_client_auth()
        };

        // Configure cipher suites based on SEV-SNP availability
        if self.use_sevsnp {
            client_config.cipher_suites = vec![
                rustls::CipherSuite::TLS13_AES_256_GCM_SHA384,
                rustls::CipherSuite::TLS13_CHACHA20_POLY1305_SHA256,
            ];
        } else {
            client_config.cipher_suites = vec![
                rustls::CipherSuite::TLS13_AES_128_GCM_SHA256,
                rustls::CipherSuite::TLS13_AES_256_GCM_SHA384,
                rustls::CipherSuite::TLS13_CHACHA20_POLY1305_SHA256,
            ];
        }

        client_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        Ok(Arc::new(client_config))
    }

    pub fn create_server_config(
        &self,
        config: &TlsConfig,
        cert_chain: Vec<rustls::Certificate>,
        key: rustls::PrivateKey,
    ) -> Result<Arc<ServerConfig>, TlsError> {
        let mut server_config = if config.verify_peer {
            ServerConfig::builder()
                .with_safe_defaults()
                .with_client_cert_verifier(Arc::new(rustls::server::AllowAnyAuthenticatedClient::new(
                    rustls::RootCertStore::empty(),
                )))
                .with_single_cert(cert_chain, key)
                .map_err(|e| TlsError::InvalidCertificate(e.to_string()))?
        } else {
            ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_single_cert(cert_chain, key)
                .map_err(|e| TlsError::InvalidCertificate(e.to_string()))?
        };

        // Configure cipher suites based on SEV-SNP availability
        if self.use_sevsnp {
            server_config.cipher_suites = vec![
                rustls::CipherSuite::TLS13_AES_256_GCM_SHA384,
                rustls::CipherSuite::TLS13_CHACHA20_POLY1305_SHA256,
            ];
        } else {
            server_config.cipher_suites = vec![
                rustls::CipherSuite::TLS13_AES_128_GCM_SHA256,
                rustls::CipherSuite::TLS13_AES_256_GCM_SHA384,
                rustls::CipherSuite::TLS13_CHACHA20_POLY1305_SHA256,
            ];
        }

        server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        Ok(Arc::new(server_config))
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm_bindings {
    use wasm_bindgen::prelude::*;
    use js_sys::Uint8Array;
    use web_sys::{WebSocket, WebSocketBinaryType};
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Mutex;
    use std::collections::VecDeque;

    #[wasm_bindgen]
    pub struct WasmTlsSocket {
        ws: WebSocket,
        context: Arc<WasmTlsContext>,
        connected: Arc<AtomicBool>,
        message_queue: Arc<Mutex<VecDeque<Vec<u8>>>>,
    }

    #[wasm_bindgen]
    impl WasmTlsSocket {
        #[wasm_bindgen(constructor)]
        pub fn new(url: &str, use_sevsnp: bool) -> Result<WasmTlsSocket, JsValue> {
            let ws = WebSocket::new(url).map_err(|e| JsValue::from_str(&format!("Failed to create WebSocket: {:?}", e)))?;
            ws.set_binary_type(WebSocketBinaryType::Arraybuffer);
            
            let connected = Arc::new(AtomicBool::new(false));
            let message_queue = Arc::new(Mutex::new(VecDeque::new()));

            let socket = WasmTlsSocket {
                ws: ws.clone(),
                context: Arc::new(WasmTlsContext { use_sevsnp }),
                connected: connected.clone(),
                message_queue: message_queue.clone(),
            };

            // Set up connection handler
            let on_open = Closure::wrap(Box::new(move || {
                connected.store(true, Ordering::SeqCst);
            }) as Box<dyn FnMut()>);
            ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
            on_open.forget();

            // Set up error handler
            let on_error = Closure::wrap(Box::new(move |event: web_sys::Event| {
                connected.store(false, Ordering::SeqCst);
                web_sys::console::error_1(&JsValue::from_str(&format!("WebSocket error: {:?}", event)));
            }) as Box<dyn FnMut(_)>);
            ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
            on_error.forget();

            // Set up close handler
            let on_close = Closure::wrap(Box::new(move || {
                connected.store(false, Ordering::SeqCst);
            }) as Box<dyn FnMut()>);
            ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
            on_close.forget();

            Ok(socket)
        }

        pub fn send(&self, data: &[u8]) -> Result<(), JsValue> {
            if !self.connected.load(Ordering::SeqCst) {
                return Err(JsValue::from_str("Not connected"));
            }

            let array = Uint8Array::from(data);
            self.ws.send_with_u8_array(&array)
        }

        pub fn set_onmessage(&self, callback: &js_sys::Function) {
            let message_queue = self.message_queue.clone();
            let ws = self.ws.clone();
            let closure = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
                if let Ok(array_buffer) = event.data().dyn_into::<js_sys::ArrayBuffer>() {
                    let array = Uint8Array::new(&array_buffer);
                    let mut queue = message_queue.lock().unwrap();
                    queue.push_back(array.to_vec());
                    let _ = callback.call1(&JsValue::null(), &array);
                }
            }) as Box<dyn FnMut(_)>);
            
            self.ws.set_onmessage(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }

        pub fn is_connected(&self) -> bool {
            self.connected.load(Ordering::SeqCst)
        }

        pub fn get_message_queue_length(&self) -> usize {
            self.message_queue.lock().unwrap().len()
        }

        pub fn pop_message(&self) -> Option<Vec<u8>> {
            self.message_queue.lock().unwrap().pop_front()
        }
    }
} 