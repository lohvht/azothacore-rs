use std::{collections::BTreeMap, io, path::PathBuf};

use proc_macro2::{Span, TokenStream};
use prost_build::{Config as ProstBuildConfig, Method, ServiceGenerator};
use quote::{format_ident, quote};
use walkdir::WalkDir;

fn service_names_to_known_hashes() -> BTreeMap<&'static str, (u32, &'static str, u32)> {
    BTreeMap::from([
        (
            "bgs.protocol.friends.v1.FriendsService",
            (0xabdfed63, "bnet.protocol.friends.FriendsService", 0xa3ddb1bd),
        ),
        (
            "bgs.protocol.friends.v1.FriendsListener",
            (0xA6717548, "bnet.protocol.friends.FriendsNotify", 0x6F259A13),
        ),
        (
            "bgs.protocol.account.v1.AccountService",
            (0x1E4DC42F, "bnet.protocol.account.AccountService", 0x62DA0891),
        ),
        (
            "bgs.protocol.account.v1.AccountListener",
            (0x7807483C, "bnet.protocol.account.AccountNotify", 0x54DFDA17),
        ),
        (
            "bgs.protocol.authentication.v1.AuthenticationListener",
            (0x4DA86228, "bnet.protocol.authentication.AuthenticationClient", 0x71240E35),
        ),
        (
            "bgs.protocol.authentication.v1.AuthenticationService",
            (0xFF5A6AC3, "bnet.protocol.authentication.AuthenticationServer", 0xDECFC01),
        ),
        (
            "bgs.protocol.channel.v1.ChannelListener",
            (0xDA660990, ("bnet.protocol.channel.ChannelSubscriber"), 0xBF8C8094),
        ),
        (
            "bgs.protocol.channel.v1.ChannelService",
            (0xA913A87B, "bnet.protocol.channel.Channel", 0xB732DB32),
        ),
        (
            "bgs.protocol.game_utilities.v1.GameUtilitiesService",
            (0x51923A28, "bnet.protocol.game_utilities.GameUtilities", 0x3FC1274D),
        ),
        (
            "bgs.protocol.user_manager.v1.UserManagerService",
            (0x8EE5694E, "bnet.protocol.user_manager.UserManagerService", 0x3E19268A),
        ),
        (
            "bgs.protocol.user_manager.v1.UserManagerListener",
            (0xB3426BB3, "bnet.protocol.user_manager.UserManagerNotify", 0xBC872C22),
        ),
        (
            "bgs.protocol.connection.v1.ConnectionService",
            (0x2782094B, "bnet.protocol.connection.ConnectionService", 0x65446991),
        ),
        (
            "bgs.protocol.presence.v1.PresenceService",
            (0xD8F94B3B, "bnet.protocol.presence.PresenceService", 0xFA0796FF),
        ),
        (
            "bgs.protocol.challenge.v1.ChallengeListener",
            (0xC6D90AB8, "bnet.protocol.challenge.ChallengeNotify", 0xBBDA171F),
        ),
        (
            "bgs.protocol.challenge.v1.ChallengeService",
            (0x71BB6833, "bnet.protocol.challenge.ChallengeService", 0xDBBF6F19),
        ),
        (
            "bgs.protocol.report.v1.ReportService",
            (0x724F5F47, "bnet.protocol.report.ReportService", 0x7CAF61C9),
        ),
        (
            "bgs.protocol.resources.v1.ResourcesService",
            (0x4B104C53, "bnet.protocol.resources.Resources", 0xECBE75BA),
        ),
    ])
}

/// implementation of FNV-1a hash. Used by bnet to get the service_hash
fn fnv_1a_hash(name: &str) -> u32 {
    let mut hash = 0x811C9DC5;
    for b in name.as_bytes() {
        hash ^= *b as u32;
        hash = hash.wrapping_mul(0x1000193);
    }

    hash
}

fn get_service_hashes(full_service_name: &str) -> (u32, u32) {
    let (name_hash, original_name, original_name_hash) = *service_names_to_known_hashes()
        .get(full_service_name)
        .unwrap_or_else(|| panic!("Service hash for '{full_service_name}' does not exist, but must be matched to some entry"));

    let calc_name_hash = fnv_1a_hash(full_service_name);
    if calc_name_hash != name_hash {
        panic!("Somehow the calculated name hash for {full_service_name} does not match the one hardcoded. This is probably our mistake and should never happen; calc={calc_name_hash}, coded={name_hash}");
    }
    let calc_original_name_hash = fnv_1a_hash(original_name);
    if calc_original_name_hash != original_name_hash {
        panic!("Somehow the calculated original name hash for {original_name} does not match the one hardcoded. This is probably our mistake and should never happen; calc={calc_original_name_hash}, coded={original_name_hash}");
    }
    (name_hash, original_name_hash)
}

pub struct BnetRpcServiceGenerator;

struct MethodInfo {
    method_id:                   u64,
    proto_name:                  String,
    server_method_name:          String,
    client_method_name:          String,
    client_response_method_name: String,
    request_type:                String,
    return_type:                 String,
}

impl MethodInfo {
    fn new(m: &Method) -> Result<Self, io::Error> {
        // retrieve method ID from trailing comments
        let trailing = m.comments.trailing.join(" ").trim().to_string();

        let method_id = if let Some(mid_str) = trailing.strip_prefix("method_id: ") {
            mid_str
                .parse()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("error parsing method_id '{mid_str}', err={e}")))
        } else {
            panic!("Method ID should be set at least on method - this step is hardcoded and taken manually from other sources (like TC) as the reverse engineered proto files don't have them" );
        };

        method_id.map(|method_id| Self {
            method_id,
            proto_name: m.proto_name.clone(),
            server_method_name: format!("handle_srv_req_{}", m.name),
            client_method_name: m.name.clone(),
            client_response_method_name: format!("handle_cli_resp_{}", m.name),
            request_type: m.input_type.clone(),
            return_type: m.output_type.clone(),
        })
    }
}

impl BnetRpcServiceGenerator {
    fn generate_service_trait_methods(service_name: &str, service_methods: &[Method]) -> Vec<TokenStream> {
        let mut service_methods_pieces = vec![];
        for m in service_methods {
            let MethodInfo {
                proto_name: proto_method_name,
                server_method_name,
                request_type,
                return_type,
                client_method_name,
                client_response_method_name,
                method_id,
            } = match MethodInfo::new(m) {
                Err(e) => {
                    let proto_method_name = &m.proto_name;
                    println!("cargo:warning=generate_service_trait_methods: parsing method info for {service_name}.{proto_method_name} ran into error: {e}");
                    continue;
                },
                Ok(i) => i,
            };
            let proto_return_type = &m.output_proto_type;
            let request_type = request_type.parse::<TokenStream>().unwrap();
            let return_type = return_type.parse::<TokenStream>().unwrap();

            let client_method_name = format_ident!("{client_method_name}");
            let method_id = syn::LitInt::new(&format!("{method_id}"), Span::call_site());
            let client_debug_message = syn::LitStr::new(
                &format!("Server called client method {service_name}.{proto_method_name}({{request:?}})"),
                Span::call_site(),
            );

            // std::function<void(::bgs::protocol::NoData const*)> responseCallback
            // std::function<void(MessageBuffer)> callback = [responseCallback](MessageBuffer buffer) -> void {
            //     ::bgs::protocol::NoData response;
            //     if (response.ParseFromArray(buffer.GetReadPointer(), buffer.GetActiveSize()))
            //       responseCallback(&response);
            //   };

            let client_response_method = if proto_return_type.contains("NO_RESPONSE") {
                quote! {}
            } else {
                let client_response_method_name = format_ident!("{client_response_method_name}");
                let client_response_debug_message = syn::LitStr::new(
                    &format!("server received response for method {service_name}.{proto_method_name}({{response:?}})"),
                    Span::call_site(),
                );
                quote! {
                    fn #client_response_method_name(&self, response: #return_type) -> impl ::std::future::Future<Output=::std::io::Result<()>> + Send {
                        async move {
                            tracing::debug!(#client_response_debug_message);
                            Ok(())
                        }
                    }
                }
            };

            let server_method_name = format_ident!("{server_method_name}");
            let server_error_msg = syn::LitStr::new(
                &format!("Client tried to call not implemented server method {service_name}.{proto_method_name}({{request:?}})"),
                Span::call_site(),
            );
            let method_piece = quote!(
                #[tracing::instrument(skip(self), target = "service.protobuf", fields(caller_info=self.caller_info()))]
                fn #client_method_name(&self, request: #request_type) -> impl ::std::future::Future<Output=::std::io::Result<()>> + Send where Self: Sync, {
                    async {
                        tracing::debug!(#client_debug_message);
                        let request = self.pre_send_store_client_request(self.service_hash(), #method_id, request).await?;
                        self.make_client_request(request).await
                    }
                }

                #client_response_method

                #[tracing::instrument(skip(self), target = "service.protobuf", fields(caller_info=self.caller_info()))]
                fn #server_method_name(&self, request: #request_type) -> impl ::std::future::Future<Output=crate::BnetRpcResult<#return_type>> + Send where Self: Sync, {
                    async {
                        tracing::error!(#server_error_msg);
                        Err(crate::BattlenetRpcErrorCode::RpcNotImplemented)
                    }
                }
            );
            service_methods_pieces.push(method_piece);
        }
        service_methods_pieces
    }

    fn generate_service_hash_method(service_name: &str, service_package: &str) -> TokenStream {
        let full_name = format!("{service_package}.{service_name}");
        let (name_hash, original_name_hash) = get_service_hashes(&full_name);
        let name_hash = syn::LitInt::new(&format!("0x{name_hash:X}"), Span::call_site());
        let original_name_hash = syn::LitInt::new(&format!("0x{original_name_hash:X}"), Span::call_site());

        quote!(
            fn service_hash(&self) -> u32 {
                if Self::USE_ORIGINAL_HASH {
                    #original_name_hash
                } else {
                    #name_hash
                }
            }
        )
    }

    fn generate_call_server_method(service_name: &str, service_methods: &[Method]) -> TokenStream {
        let mut call_service_block_pieces = vec![];
        for m in service_methods {
            let MethodInfo {
                proto_name: proto_method_name,
                server_method_name: rs_method_name,
                request_type,
                method_id,
                ..
            } = match MethodInfo::new(m) {
                Err(e) => {
                    let proto_method_name = &m.proto_name;
                    println!("cargo:warning=generate_call_server_method: parsing method info for {service_name}.{proto_method_name} ran into error: {e}");
                    continue;
                },
                Ok(i) => i,
            };
            let method_id = syn::LitInt::new(&format!("{method_id}"), Span::call_site());
            let request_type = request_type.parse::<TokenStream>().unwrap();
            let proto_return_type = &m.output_proto_type;

            let rs_method_name = format_ident!("{}", rs_method_name);
            let parse_error_msg = syn::LitStr::new(
                &format!("Failed to parse request for {service_name}.{proto_method_name} server method call. err={{e}}"),
                Span::call_site(),
            );
            let debug_message_before = syn::LitStr::new(
                &format!("Client called server method {service_name}.{proto_method_name}({{request:?}})."),
                Span::call_site(),
            );
            let debug_message_after = syn::LitStr::new(
                &format!("Client called server method {service_name}.{proto_method_name} returned. result={{result:?}}"),
                Span::call_site(),
            );

            let return_block = if proto_return_type.contains("NO_RESPONSE") {
                quote!(
                    if result.is_ok() {
                        // no response has no error => No need to send anything back
                        return Ok(());
                    }
                    self.send_server_response(crate::BnetServiceWrapper {
                        service_hash: self.service_hash(),
                        method_id,
                        token,
                        result,
                    })
                    .await
                )
            } else {
                quote!(
                    self.send_server_response(crate::BnetServiceWrapper {
                        service_hash: self.service_hash(),
                        method_id,
                        token,
                        result,
                    })
                    .await
                )
            };

            call_service_block_pieces.push(quote! {
                #method_id => {
                    match #request_type::decode(message) {
                        Err(e) => {
                            tracing::debug!(#parse_error_msg);
                            self.send_server_response::<()>(crate::BnetServiceWrapper {
                                service_hash: self.service_hash(),
                                method_id,
                                token,
                                result: Err(crate::BattlenetRpcErrorCode::RpcMalformedRequest),
                            })
                            .await
                        },
                        Ok(request) => {
                            tracing::debug!(#debug_message_before);
                            let result = self.#rs_method_name(request).await;
                            tracing::debug!(#debug_message_after);

                            #return_block
                        },
                    }
                },
            });
        }

        quote! {
            #[tracing::instrument(skip(self, message), target = "service.protobuf", fields(caller_info=self.caller_info()))]
            fn call_server_method<B>(&self, token: u32, method_id: u32, message: B) -> impl ::std::future::Future<Output=::std::io::Result<()>> + Send
            where
                Self: Sync,
                B: bytes::Buf + Send,
            {
                use prost::Message;
                async {
                    match method_id {
                        #(
                            #call_service_block_pieces
                        )*
                        m => {
                            tracing::error!("Bad method id {}.", m);
                            self.send_server_response::<()>(crate::BnetServiceWrapper {
                                service_hash: self.service_hash(),
                                method_id,
                                token,
                                result: Err(crate::BattlenetRpcErrorCode::RpcInvalidMethod),
                            })
                            .await
                        },
                    }
                }
            }
        }
    }

    fn generate_receive_client_response_method(service_name: &str, service_methods: &[Method]) -> TokenStream {
        let mut receive_client_response_pieces = vec![];
        for m in service_methods {
            let MethodInfo {
                proto_name: proto_method_name,
                return_type,
                method_id,
                client_response_method_name,
                ..
            } = match MethodInfo::new(m) {
                Err(e) => {
                    let proto_method_name = &m.proto_name;
                    println!("cargo:warning=generate_call_server_method: parsing method info for {service_name}.{proto_method_name} ran into error: {e}");
                    continue;
                },
                Ok(i) => i,
            };
            let proto_return_type = &m.output_proto_type;
            if proto_return_type.contains("NO_RESPONSE") {
                continue;
            }
            let method_id = syn::LitInt::new(&format!("{method_id}"), Span::call_site());
            let return_type = return_type.parse::<TokenStream>().unwrap();

            let cli_resp_method_name = format_ident!("{client_response_method_name}");
            let parse_error_msg = syn::LitStr::new(
                &format!("Failed to parse response from client for {service_name}.{proto_method_name} method call. err={{e}}"),
                Span::call_site(),
            );
            let debug_message = syn::LitStr::new(
                &format!("server called client method {service_name}.{proto_method_name}({{response:?}})."),
                Span::call_site(),
            );
            receive_client_response_pieces.push(quote! {
                #method_id => {
                    match #return_type::decode(_message) {
                        Err(e) => {
                            tracing::debug!(#parse_error_msg);
                            Ok(())
                        },
                        Ok(response) => {
                            tracing::debug!(#debug_message);
                            self.#cli_resp_method_name(response).await
                        },
                    }
                },
            });
        }

        let receive_client_response_body = if receive_client_response_pieces.is_empty() {
            quote!(Ok(()))
        } else {
            quote!(
                use prost::Message;
                match _method_id {
                    #(
                        #receive_client_response_pieces
                    )*
                    m => {
                        tracing::warn!("Bad method id {} for a client side response", m);
                        Ok(())
                    },
                }
            )
        };

        quote! {
            #[tracing::instrument(skip(self, _message), target = "service.protobuf", fields(caller_info=self.caller_info()))]
            fn receive_client_response<B>(&self, _method_id: u32, _message: B) -> impl ::std::future::Future<Output=::std::io::Result<()>> + Send
            where
                Self: Sync,
                B: bytes::Buf + Send,
            {
                async {
                    #receive_client_response_body
                }
            }
        }
    }

    fn generate_service_trait(&self, service_name: &str, service_package: &str, service_methods: &[Method], buf: &mut String) {
        let server_service_methods_pieces = Self::generate_service_trait_methods(service_name, service_methods);

        let service_hash_method = Self::generate_service_hash_method(service_name, service_package);
        let service_call_server_method = Self::generate_call_server_method(service_name, service_methods);

        let receive_client_response = Self::generate_receive_client_response_method(service_name, service_methods);

        let service_server_name = format_ident!("{service_name}");
        let service_definiton = quote!(
            pub trait #service_server_name: crate::BnetRpcService {
                const USE_ORIGINAL_HASH: bool;

                #service_hash_method

                #service_call_server_method

                #receive_client_response

                #(
                    #server_service_methods_pieces
                )*
            }
        )
        .to_string();
        buf.push_str(&service_definiton);
    }
}

impl ServiceGenerator for BnetRpcServiceGenerator {
    fn generate(&mut self, service: prost_build::Service, buf: &mut String) {
        self.generate_service_trait(&service.name, &service.package, &service.methods, buf);
        println!("/// OUTPUT ++> \n{buf}");
        //
    }
}

fn main() -> io::Result<()> {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=protos");

    // use a vendored protoc
    std::env::set_var("PROTOC", protobuf_src::protoc());

    let service_generator = BnetRpcServiceGenerator {};

    let protos = WalkDir::new("protos/current")
        .into_iter()
        .filter_map(|de| {
            de.ok().and_then(|e| {
                let p = e.into_path();
                if p.is_file() {
                    Some(p)
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    let includes: Vec<PathBuf> = vec![];

    ProstBuildConfig::new()
        .btree_map(["."])
        .bytes(["."])
        .include_file("_includes.rs")
        .protoc_arg("--proto_path=protos/current")
        .service_generator(Box::new(service_generator))
        .compile_protos(&protos, &includes)?;

    Ok(())
}
