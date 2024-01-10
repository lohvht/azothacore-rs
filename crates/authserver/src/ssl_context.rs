use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
    sync::OnceLock,
};

use azothacore_common::{configuration::ConfigMgr, AzResult, CONF_DIR};
use tokio_native_tls::{
    native_tls::{self, Identity},
    TlsAcceptor,
};
use tracing::{debug, error};

pub struct SslContext;

impl SslContext {
    pub fn initialise() -> AzResult<()> {
        fn helper() -> AzResult<TlsAcceptor> {
            let certificate_chain_file = Path::new(CONF_DIR).join(ConfigMgr::r().get("CertificatesFile", || "bnetserver.cert.pem".to_string()));
            let private_key_file = Path::new(CONF_DIR).join(ConfigMgr::r().get("PrivateKeyFile", || "bnetserver.key.pem".to_string()));

            debug!(target:"server::authserver", cert=%certificate_chain_file.display(), privkey=%private_key_file.display(), "Attempting to open cert and private key files");
            let mut cert_chain = vec![];
            BufReader::new(File::open(certificate_chain_file)?).read_to_end(&mut cert_chain)?;
            let mut key_der = vec![];
            BufReader::new(File::open(private_key_file)?).read_to_end(&mut key_der)?;

            Ok(TlsAcceptor::from(native_tls::TlsAcceptor::new(Identity::from_pkcs8(&cert_chain, &key_der)?)?))
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
