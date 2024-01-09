pub mod data_stores;
pub mod networking;
pub mod realms;
pub mod secrets;
pub mod shared_defines;

use std::{
    future::Future,
    io::{self, Write},
    sync::{mpsc, Arc, Mutex},
};

use azothacore_common::AzResult;
use tokio_util::sync::CancellationToken;
use tracing::debug;

pub struct DropperWrapperFn {
    cancel_token:              CancellationToken,
    has_finished_cancellation: Mutex<mpsc::Receiver<()>>,
}

impl Drop for DropperWrapperFn {
    fn drop(&mut self) {
        self.cancel_token.cancel();
        _ = self.has_finished_cancellation.lock().unwrap().recv();
    }
}

/// dropper_wrapper_fn provides an easy way to properly handled by awaiting
/// any futures passed into after cancelling the given token on drop
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

/// NOTE: This matches the zlib use for bnetrpc in TC/AC code, check if this is strict though
/// Could be possible to use a higher level impl
// pub fn bnetrpc_zcompress(json: Vec<u8>) -> Option<Vec<u8>> {
//     use libz_sys::{compress, compressBound, Z_OK};
//     let mut compressed_length = unsafe { compressBound(json.len() as u64) };
//     let mut compressed = vec![0; compressed_length as usize + 4];
//     let payload_eventual_size_in_bytes = u32::try_from(json.len() + 1).unwrap().to_le_bytes();
//     compressed[..4].clone_from_slice(&payload_eventual_size_in_bytes);

//     if unsafe { compress(compressed[4..].as_mut_ptr(), &mut compressed_length, json.as_ptr(), json.len() as u64 + 1) == Z_OK } {
//         compressed.resize(compressed_length as usize, 0);
//         Some(compressed)
//     } else {
//         None
//     }
// }

pub fn bnetrpc_zcompress(json: Vec<u8>) -> io::Result<Vec<u8>> {
    use flate2::{write::ZlibEncoder, Compression};
    // use libz_sys::{compress, compressBound, deflate};

    // Starts with the total eventual size
    let mut e = ZlibEncoder::new(Vec::from(u32::try_from(json.len() + 1).unwrap().to_le_bytes()), Compression::default());
    e.write_all(&json)?;
    e.flush_finish()
}
