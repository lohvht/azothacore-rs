pub mod data_stores;
pub mod networking;
pub mod realms;
pub mod secrets;
pub mod shared_defines;

use std::future::Future;

use azothacore_common::AzResult;
use libz_sys::{compress, compressBound, Z_OK};
use tracing::error;

pub struct DropperWrapperFn<F, Fut>
where
    F: Fn() -> Fut,
    Fut: Future<Output = AzResult<()>>,
{
    f: F,
    h: tokio::runtime::Handle,
}

pub fn dropper_wrapper_fn<F, Fut>(h: &tokio::runtime::Handle, f: F) -> DropperWrapperFn<F, Fut>
where
    F: Fn() -> Fut,
    Fut: Future<Output = AzResult<()>>,
{
    DropperWrapperFn { f, h: h.clone() }
}

impl<F, Fut> Drop for DropperWrapperFn<F, Fut>
where
    F: Fn() -> Fut,
    Fut: Future<Output = AzResult<()>>,
{
    fn drop(&mut self) {
        if let Err(e) = self.h.block_on((self.f)()) {
            error!("Error when attempting to run the drop callback: err: {e}");
        };
    }
}

pub fn bnetrpc_zcompress(json: Vec<u8>) -> Option<Vec<u8>> {
    let mut compressed_length = unsafe { compressBound(json.len() as u64) };
    let mut compressed = vec![0; compressed_length as usize + 4];
    let payload_eventual_size_in_bytes = u32::try_from(json.len() + 1).unwrap().to_le_bytes();
    compressed[..4].clone_from_slice(&payload_eventual_size_in_bytes);

    if unsafe { compress(compressed[4..].as_mut_ptr(), &mut compressed_length, json.as_ptr(), json.len() as u64 + 1) == Z_OK } {
        compressed.resize(compressed_length as usize, 0);
        Some(compressed)
    } else {
        None
    }
}
