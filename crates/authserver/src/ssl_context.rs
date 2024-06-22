use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

use azothacore_common::{bevy_app::AzStartupFailedEvent, configuration::ConfigMgr, deref_boilerplate, AzResult, CONF_DIR};
use bevy::prelude::*;
use tokio_native_tls::{
    native_tls::{self, Identity},
    TlsAcceptor,
};
use tracing::debug;

#[derive(Resource, Clone)]
pub struct SslContext(Arc<TlsAcceptor>);

deref_boilerplate!(SslContext, Arc<TlsAcceptor>, 0);

impl SslContext {
    pub fn new(certificates_file: PathBuf, private_key_file: PathBuf) -> AzResult<SslContext> {
        let certificate_chain_file = Path::new(CONF_DIR).join(certificates_file);
        let private_key_file = Path::new(CONF_DIR).join(private_key_file);

        debug!(target:"server::authserver", cert=%certificate_chain_file.display(), privkey=%private_key_file.display(), "Attempting to open cert and private key files");
        let mut cert_chain = vec![];
        BufReader::new(File::open(certificate_chain_file)?).read_to_end(&mut cert_chain)?;
        let mut key_der = vec![];
        BufReader::new(File::open(private_key_file)?).read_to_end(&mut key_der)?;

        Ok(SslContext(Arc::new(TlsAcceptor::from(native_tls::TlsAcceptor::new(Identity::from_pkcs8(
            &cert_chain,
            &key_der,
        )?)?))))
    }
}

pub trait SslContextConfig: Send + Sync + 'static {
    fn certs_file(&self) -> PathBuf;
    fn privkey_file(&self) -> PathBuf;
}

fn init_ssl_context<C: SslContextConfig>(mut commands: Commands, cfg: Res<ConfigMgr<C>>, mut ev_startup_failed: EventWriter<AzStartupFailedEvent>) {
    let ssl_context = match SslContext::new(cfg.certs_file(), cfg.privkey_file()) {
        Err(e) => {
            ev_startup_failed.send_default();
            error!(cause=%e, "error initialising SSL context");
            return;
        },
        Ok(s) => s,
    };
    commands.insert_resource(ssl_context);
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SetSslContextSet;

pub fn ssl_context_plugin<C: SslContextConfig>(app: &mut App) {
    app.add_systems(Startup, init_ssl_context::<C>.run_if(resource_exists::<ConfigMgr<C>>).in_set(SetSslContextSet));
}
