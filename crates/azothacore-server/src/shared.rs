pub mod data_stores;
pub mod networking;
pub mod realms;
pub mod secrets;
pub mod shared_defines;

use std::{
    future::Future,
    io::{self, Write},
    sync::{mpsc, Mutex},
};

use azothacore_common::AzResult;
use tokio_util::sync::CancellationToken;
use tracing::debug;

pub struct DropperWrapperFn {
    cancel_token:              CancellationToken,
    /// Need a mutex to ensure that mpsc is sync. Its pretty much the only way to bridge the
    /// async to sync boundary
    has_finished_cancellation: Mutex<mpsc::Receiver<()>>,
}

impl Drop for DropperWrapperFn {
    fn drop(&mut self) {
        self.cancel_token.cancel();
        _ = self.has_finished_cancellation.lock().unwrap().recv();
    }
}

/// dropper_wrapper_fn provides an easy way to properly handle async futures
/// by awaiting any futures passed into it on drop.
///
/// It handles this by cancelling the given token on drop, this causes
/// the spawned inner handler to unblock and causes the underlying future to
/// be awaited before sending the done signal back.
///
pub fn dropper_wrapper_fn<Fut>(handler: &tokio::runtime::Handle, cancel_token: CancellationToken, f: Fut) -> DropperWrapperFn
where
    Fut: Future<Output = AzResult<()>> + Send + 'static,
{
    let (s, r) = mpsc::channel();
    let ct = cancel_token.clone();

    handler.spawn(async move {
        ct.cancelled().await;

        if let Err(e) = f.await {
            debug!(cause=%e, "error when cleaning up async function");
        }
        _ = s.send(());
    });
    let has_finished_cancellation = Mutex::new(r);

    DropperWrapperFn {
        cancel_token,
        has_finished_cancellation,
    }
}

pub fn bnetrpc_zcompress(mut json: Vec<u8>) -> io::Result<Vec<u8>> {
    use flate2::{write::ZlibEncoder, Compression};
    json.push(b'\0');
    let mut compressed = u32::try_from(json.len()).unwrap().to_le_bytes().to_vec();

    // Starts with the total eventual size
    let mut e = ZlibEncoder::new(vec![], Compression::default());
    e.write_all(&json)?;
    let mut res = e.finish()?;
    compressed.append(&mut res);
    Ok(compressed)
}
