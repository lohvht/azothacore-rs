pub mod data_stores;
pub mod realms;
pub mod secrets;
pub mod shared_defines;

use std::future::Future;

use azothacore_common::AzResult;
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
