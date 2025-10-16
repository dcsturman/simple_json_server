/// TLS configuration for secure connections
///
/// # Example
///
/// ```rust
/// use simple_json_server::TlsConfig;
///
/// let tls_config = TlsConfig::new("cert.pem", "key.pem");
/// ```
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Path to the certificate file (PEM format)
    pub cert_path: String,
    /// Path to the private key file (PEM format)
    pub key_path: String,
}

impl TlsConfig {
    /// Create a new TLS configuration
    pub fn new(cert_path: impl Into<String>, key_path: impl Into<String>) -> Self {
        Self {
            cert_path: cert_path.into(),
            key_path: key_path.into(),
        }
    }

    /// Load the TLS configuration and create a rustls ServerConfig
    pub(crate) async fn load_server_config(
        &self,
    ) -> Result<rustls::ServerConfig, Box<dyn std::error::Error + Send + Sync>> {
        use rustls_pemfile::{certs, pkcs8_private_keys};
        use std::io::BufReader;
        use tokio::fs::File;
        use tokio::io::AsyncReadExt;

        // Read certificate file
        let mut cert_file = File::open(&self.cert_path).await?;
        let mut cert_data = Vec::new();
        cert_file.read_to_end(&mut cert_data).await?;
        let mut cert_reader = BufReader::new(cert_data.as_slice());
        let cert_chain: Vec<rustls::pki_types::CertificateDer> =
            certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;

        // Read private key file
        let mut key_file = File::open(&self.key_path).await?;
        let mut key_data = Vec::new();
        key_file.read_to_end(&mut key_data).await?;
        let mut key_reader = BufReader::new(key_data.as_slice());
        let mut keys: Vec<rustls::pki_types::PrivateKeyDer> = pkcs8_private_keys(&mut key_reader)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(rustls::pki_types::PrivateKeyDer::Pkcs8)
            .collect();

        if keys.is_empty() {
            return Err("No private key found".into());
        }

        let private_key = keys.remove(0);

        // Create server config
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, private_key)?;

        Ok(config)
    }
}
