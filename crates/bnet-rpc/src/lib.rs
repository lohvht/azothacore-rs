use std::io;

include!(concat!(env!("OUT_DIR"), "/_includes.rs"));

mod rpc_error_codes;

pub use rpc_error_codes::{BattlenetRpcErrorCode, BnetRpcResult};

pub struct BnetServiceWrapper<M>
where
    M: prost::Message,
{
    pub service_hash: u32,
    pub method_id:    u32,
    pub token:        u32,
    pub result:       BnetRpcResult<M>,
}

pub trait BnetRpcService {
    /// The Bnet session caller's information
    fn caller_info(&self) -> String;

    /// The response from server side (i.e. Azocore)
    fn send_server_response<M>(&self, response: BnetServiceWrapper<M>) -> impl std::future::Future<Output = io::Result<()>> + Send
    where
        M: prost::Message;

    /// Called before actually making the client request via `make_client_request`
    /// This should be overwritten to provide a reference to "callback" later
    /// (via an implementation) of a server method if such a behaviour is desired.
    ///
    fn pre_send_store_client_request<M>(
        &self,
        service_hash: u32,
        method_id: u32,
        request: M,
    ) -> impl std::future::Future<Output = io::Result<BnetServiceWrapper<M>>> + Send
    where
        M: prost::Message;

    /// The request that we're making to the client side (i.e. Wow clients connecting to Azocore)
    fn make_client_request<M>(&self, request: BnetServiceWrapper<M>) -> impl std::future::Future<Output = io::Result<()>> + Send
    where
        M: prost::Message;
}
