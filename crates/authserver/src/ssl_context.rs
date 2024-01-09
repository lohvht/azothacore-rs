use std::{
    fs::File,
    io::BufReader,
    path::Path,
    sync::{Arc, OnceLock},
};

use azothacore_common::{configuration::ConfigMgr, AzResult, CONF_DIR};
use rustls_pemfile::{certs, rsa_private_keys};
use tokio_rustls::{
    rustls::{pki_types::PrivateKeyDer, version::TLS12, ServerConfig},
    TlsAcceptor,
};
use tracing::{debug, error};

pub struct SslContext;

impl SslContext {
    pub fn initialise() -> AzResult<()> {
        fn helper() -> AzResult<TlsAcceptor> {
            let builder = ServerConfig::builder_with_protocol_versions(&[&TLS12]).with_no_client_auth();

            let certificate_chain_file = Path::new(CONF_DIR).join(ConfigMgr::r().get("CertificatesFile", || "bnetserver.cert.pem".to_string()));
            let private_key_file = Path::new(CONF_DIR).join(ConfigMgr::r().get("PrivateKeyFile", || "bnetserver.key.pem".to_string()));

            debug!(target:"server::authserver", cert=%certificate_chain_file.display(), privkey=%private_key_file.display(), "Attempting to open cert and private key files");
            let cert_chain = certs(&mut BufReader::new(File::open(certificate_chain_file)?)).filter_map(|v| v.ok()).collect();

            let key_der = rsa_private_keys(&mut BufReader::new(File::open(private_key_file)?))
                .filter_map(|v| v.ok())
                .next()
                .unwrap();
            let cfg = builder.with_single_cert(cert_chain, PrivateKeyDer::Pkcs1(key_der))?;

            Ok(TlsAcceptor::from(Arc::new(cfg)))
        }

        let acceptor = helper().map_err(|e| {
            error!(target:"server::authserver", cause=%e, "Failed to initialise SSL context");
            e
        })?;

        SSL_CONTEXT.get_or_init(move || acceptor);

        Ok(())
    }

    pub fn get() -> &'static TlsAcceptor {
        SSL_CONTEXT.get().expect("SSL context not initialised")
    }
}

static SSL_CONTEXT: OnceLock<TlsAcceptor> = OnceLock::new();
