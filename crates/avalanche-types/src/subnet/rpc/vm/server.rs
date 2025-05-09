//! RPC Chain VM Server.
use std::{sync::Arc, time::Duration};

use crate::{
    ids,
    proto::pb::{
        self,
        aliasreader::alias_reader_client::AliasReaderClient,
        google::protobuf::Empty,
        keystore::keystore_client::KeystoreClient,
        messenger::{messenger_client::MessengerClient, NotifyRequest},
        sharedmemory::shared_memory_client::SharedMemoryClient,
        vm,
    },
    subnet::rpc::{
        consensus::snowman::{Block, Decidable},
        context::Context,
        database::rpcdb::{client::DatabaseClient, error_to_error_code},
        database::{corruptabledb, manager::DatabaseManager},
        errors,
        http::server::Server as HttpServer,
        snow::{
            engine::common::{appsender::client::AppSenderClient, message::Message},
            validators::client::ValidatorStateClient,
            State,
        },
        snowman::block::ChainVm,
        utils::{
            self,
            grpc::{self, timestamp_from_time},
        },
    },
};
use chrono::{TimeZone, Utc};
use pb::vm::vm_server::Vm;

use prost::bytes::Bytes;
use tokio::sync::{broadcast, mpsc, RwLock};

use std::time::Instant;
use tonic::{Request, Response};

pub struct Server<V> {
    /// Underlying Vm implementation.
    pub vm: Arc<RwLock<V>>,

    #[cfg(feature = "subnet_metrics")]
    #[cfg_attr(docsrs, doc(cfg(feature = "subnet_metrics")))]
    /// Subnet Prometheus process metrics.
    pub process_metrics: Arc<RwLock<prometheus::Registry>>,

    /// Stop channel broadcast producer.
    pub stop_ch: broadcast::Sender<()>,
}

impl<V: ChainVm> Server<V> {
    pub fn new(vm: V, stop_ch: broadcast::Sender<()>) -> Self {
        Self {
            vm: Arc::new(RwLock::new(vm)),
            #[cfg(feature = "subnet_metrics")]
            #[cfg_attr(docsrs, doc(cfg(feature = "subnet_metrics")))]
            process_metrics: Arc::new(RwLock::new(prometheus::default_registry().to_owned())),
            stop_ch,
        }
    }

    /// Attempts to get the ancestors of a block from the underlying Vm.
    ///
    /// # Errors
    /// 如果底层 VM 查询失败，返回 `io::Error`。
    pub async fn vm_ancestors<'a>(
        &'a self,
        block_id_bytes: &'a [u8],
        max_block_num: i32,
        max_block_size: i32,
        max_block_retrival_time: Duration,
    ) -> std::io::Result<Vec<Bytes>>
    where
        V: std::marker::Sync,
    {
        let inner_vm = self.vm.read().await;
        inner_vm
            .get_ancestors(
                ids::Id::from_slice(block_id_bytes),
                max_block_num,
                max_block_size,
                max_block_retrival_time,
            )
            .await
    }
}

#[tonic::async_trait]
impl<V> Vm for Server<V>
where
    V: ChainVm<
            DatabaseManager = DatabaseManager,
            AppSender = AppSenderClient,
            ValidatorState = ValidatorStateClient,
        > + Send
        + Sync
        + 'static,
{
    // Cross-chain methods have been removed from the VM trait
    // These methods are now handled by the app_request, app_request_failed, and app_response methods
    /// Implements "avalanchego/vms/rpcchainvm#VMServer.Initialize".
    /// ref. <https://github.com/ava-labs/avalanchego/blob/v1.11.1/vms/rpcchainvm/vm_server.go#L98>
    /// ref. <https://github.com/ava-labs/avalanchego/blob/v1.11.1/vms/rpcchainvm/vm_client.go#L123-L133>
    #[allow(clippy::too_many_lines)]
    async fn initialize(
        &self,
        req: Request<vm::InitializeRequest>,
    ) -> std::result::Result<Response<vm::InitializeResponse>, tonic::Status> {
        log::info!("initialize called");

        let req = req.into_inner();

        let db_server_addr = req.db_server_addr.as_str();
        // 合并 db_client_conn 的声明和唯一用途，防止提前 drop
        let db = corruptabledb::Database::new_boxed(DatabaseClient::new_boxed(
            utils::grpc::default_client(db_server_addr)?
                .connect()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(format!(
                        "failed to create db client conn from: {db_server_addr}: {e}",
                    ))
                })?,
        ));
        let db = db; // 移除错误的.await

        let server_addr = req.server_addr.as_str();
        let client_conn = utils::grpc::default_client(server_addr)?
            .connect()
            .await
            .map_err(|e| {
                tonic::Status::unknown(format!(
                    "failed to create client conn from: {server_addr}: {e}",
                ))
            })?;

        // Multiplexing in tonic is done by cloning the client which is very cheap.
        // ref. https://docs.rs/tonic/latest/tonic/transport/struct.Channel.html#multiplexing-requests
        let mut message = MessengerClient::new(client_conn.clone());
        let keystore = KeystoreClient::new(client_conn.clone());
        let shared_memory = SharedMemoryClient::new(client_conn.clone());
        let bc_lookup = AliasReaderClient::new(client_conn.clone());

        // 合并 ctx 的声明和唯一用途，避免提前 drop。
        let (tx_engine, mut rx_engine): (mpsc::Sender<Message>, mpsc::Receiver<Message>) =
            mpsc::channel(100);
        tokio::spawn(async move {
            loop {
                if let Some(msg) = rx_engine.recv().await {
                    log::debug!("message received: {msg:?}");
                    let _ = message
                        .notify(NotifyRequest {
                            message: msg as i32,
                        })
                        .await
                        .map_err(|s| tonic::Status::unknown(s.to_string()));
                    continue;
                }

                log::error!("engine receiver closed unexpectedly");
                return tonic::Status::unknown("engine receiver closed unexpectedly");
            }
        });

        // 合并 ctx 的声明和 initialize 调用，防止提前 drop
        // inner_vm 显式 drop，防止提前释放锁
        // 合并 inner_vm 的声明和唯一用途，防止提前 drop
        self.vm
            .write()
            .await
            .initialize(
                Some(Context {
                    network_id: req.network_id,
                    subnet_id: ids::Id::from_slice(&req.subnet_id),
                    chain_id: ids::Id::from_slice(&req.chain_id),
                    node_id: ids::node::Id::from_slice(&req.node_id),
                    x_chain_id: ids::Id::from_slice(&req.x_chain_id),
                    c_chain_id: ids::Id::from_slice(&req.c_chain_id),
                    avax_asset_id: ids::Id::from_slice(&req.avax_asset_id),
                    keystore,
                    shared_memory,
                    bc_lookup,
                    chain_data_dir: req.chain_data_dir,
                    validator_state: ValidatorStateClient::new(client_conn.clone()),
                }),
                db,
                &req.genesis_bytes,
                &req.upgrade_bytes,
                &req.config_bytes,
                tx_engine,
                &[()],
                AppSenderClient::new(client_conn.clone()),
            )
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        // Get last accepted block on the chain
        let (last_accepted, last_accepted_block) = {
            let inner_vm = self.vm.write().await;
            let last_accepted = inner_vm.last_accepted().await?;
            let last_accepted_block = inner_vm
                .get_block(last_accepted)
                .await
                .map_err(|e| tonic::Status::unknown(e.to_string()))?;
            drop(inner_vm);
            (last_accepted, last_accepted_block)
        };

        log::debug!("last_accepted_block id: {last_accepted:?}");

        Ok(Response::new(vm::InitializeResponse {
            last_accepted_id: Bytes::from(last_accepted.to_vec()),
            last_accepted_parent_id: Bytes::from(last_accepted_block.parent().await.to_vec()),
            bytes: Bytes::from(last_accepted_block.bytes().await.to_vec()),
            height: last_accepted_block.height().await,
            timestamp: Some(timestamp_from_time(
                &Utc.timestamp_opt(
                    i64::try_from(last_accepted_block.timestamp().await).map_err(|_| {
                        std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "timestamp out of range for i64",
                        )
                    })?,
                    0,
                )
                .unwrap(),
            )),
        }))
    }

    #[allow(clippy::too_many_lines)]
    async fn shutdown(
        &self,
        _req: tonic::Request<pb::google::protobuf::Empty>,
    ) -> std::result::Result<tonic::Response<pb::google::protobuf::Empty>, tonic::Status> {
        log::debug!("shutdown called");

        // notify all gRPC servers to shutdown
        self.stop_ch
            .send(())
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(tonic::Response::new(pb::google::protobuf::Empty {}))
    }

    /// Implements "avalanchego/vms/rpcchainvm#VMServer.CreateHandlers".
    /// ref. <https://github.com/ava-labs/avalanchego/blob/v1.11.1/vms/rpcchainvm/vm_server.go#L312-L336>
    /// ref. <https://github.com/ava-labs/avalanchego/blob/v1.11.1/vms/rpcchainvm/vm_client.go#L354-L371>
    ///
    /// Creates the HTTP handlers for custom chain network calls.
    /// This creates and exposes handlers that the outside world can use to communicate
    /// with the chain. Each handler has the path:
    /// `\[Address of node]/ext/bc/[chain ID]/[extension\]`
    ///
    /// Returns a mapping from \[extension\]s to HTTP handlers.
    /// Each extension can specify how locking is managed for convenience.
    ///
    /// For example, if this VM implements an account-based payments system,
    /// it have an extension called `accounts`, where clients could get
    /// information about their accounts.
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::too_many_lines)]
    async fn create_handlers(
        &self,
        _req: Request<Empty>,
    ) -> std::result::Result<Response<vm::CreateHandlersResponse>, tonic::Status> {
        log::debug!("create_handlers called");

        // get handlers from underlying vm
        // 合并 handlers 的声明和唯一用途，防止提前 drop
        let handlers = {
            let mut inner_vm = self.vm.write().await;
            let h = inner_vm
                .create_handlers()
                .await
                .map_err(|e| tonic::Status::unknown(format!("failed to create handlers: {e}")))?;
            drop(inner_vm);
            h
        };

        // create and start gRPC server serving HTTP service for each handler
        let mut resp_handlers: Vec<vm::Handler> = Vec::with_capacity(handlers.keys().len());
        for (prefix, http_handler) in handlers {
            let server_addr = utils::new_socket_addr();
            let server = grpc::Server::new(server_addr, self.stop_ch.subscribe());

            server
                .serve(pb::http::http_server::HttpServer::new(HttpServer::new(
                    http_handler.handler,
                )))
                .map_err(|e| {
                    tonic::Status::unknown(format!("failed to create http service: {e}"))
                })?;

            let resp_handler = vm::Handler {
                prefix,
                server_addr: server_addr.to_string(),
            };
            resp_handlers.push(resp_handler);
        }

        Ok(Response::new(vm::CreateHandlersResponse {
            handlers: resp_handlers,
        }))
    }

    async fn build_block(
        &self,
        _req: Request<vm::BuildBlockRequest>,
    ) -> std::result::Result<Response<vm::BuildBlockResponse>, tonic::Status> {
        log::debug!("build_block called");

        let block = self
            .vm
            .write()
            .await
            .build_block()
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(vm::BuildBlockResponse {
            id: Bytes::from(block.id().await.to_vec()),
            parent_id: Bytes::from(block.parent().await.to_vec()),
            bytes: Bytes::from(block.bytes().await.to_vec()),
            height: block.height().await,
            timestamp: Some(timestamp_from_time(
                &Utc.timestamp_opt(
                    i64::try_from(block.timestamp().await).unwrap_or_default(),
                    0,
                )
                .unwrap(),
            )),
            verify_with_context: false,
        }))
    }

    async fn parse_block(
        &self,
        req: Request<vm::ParseBlockRequest>,
    ) -> std::result::Result<Response<vm::ParseBlockResponse>, tonic::Status> {
        log::debug!("parse_block called");

        let req = req.into_inner();
        let block = self
            .vm
            .write()
            .await
            .parse_block(req.bytes.as_ref())
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(vm::ParseBlockResponse {
            id: Bytes::from(block.id().await.to_vec()),
            parent_id: Bytes::from(block.parent().await.to_vec()),
            height: block.height().await,
            timestamp: Some(timestamp_from_time(
                &Utc.timestamp_opt(
                    i64::try_from(block.timestamp().await).unwrap_or_default(),
                    0,
                )
                .unwrap(),
            )),
            verify_with_context: false,
        }))
    }

    /// Attempt to load a block.
    ///
    /// If the block does not exist, an empty GetBlockResponse is returned with
    /// an error code.
    ///
    /// It is expected that blocks that have been successfully verified should be
    /// returned correctly. It is also expected that blocks that have been
    /// accepted by the consensus engine should be able to be fetched. It is not
    /// required for blocks that have been rejected by the consensus engine to be
    /// able to be fetched.
    ///
    /// ref: <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/engine/snowman/block#Getter>
    async fn get_block(
        &self,
        req: Request<vm::GetBlockRequest>,
    ) -> std::result::Result<Response<vm::GetBlockResponse>, tonic::Status> {
        log::debug!("get_block called");

        let req = req.into_inner();
        let inner_vm = self.vm.read().await;

        // determine if response is an error or not
        match inner_vm.get_block(ids::Id::from_slice(&req.id)).await {
            Ok(block) => Ok(Response::new(vm::GetBlockResponse {
                parent_id: Bytes::from(block.parent().await.to_vec()),
                bytes: Bytes::from(block.bytes().await.to_vec()),
                height: block.height().await,
                timestamp: Some(timestamp_from_time(
                    &Utc.timestamp_opt(
                        i64::try_from(block.timestamp().await).unwrap_or_default(),
                        0,
                    )
                    .unwrap(),
                )),
                err: 0,
                verify_with_context: false,
            })),
            // if an error was found, generate empty response with ErrNotFound code
            // ref: https://github.com/ava-labs/avalanchego/blob/master/vms/
            Err(e) => {
                log::debug!("Error getting block");
                Ok(Response::new(vm::GetBlockResponse {
                    parent_id: Bytes::new(),
                    bytes: Bytes::new(),
                    height: 0,
                    timestamp: Some(timestamp_from_time(&Utc.timestamp_opt(0, 0).unwrap())),
                    err: error_to_error_code(&e.to_string()),
                    verify_with_context: false,
                }))
            }
        }
    }

    async fn set_state(
        &self,
        req: Request<vm::SetStateRequest>,
    ) -> std::result::Result<Response<vm::SetStateResponse>, tonic::Status> {
        log::debug!("set_state called");

        let req = req.into_inner();
        let state = State::try_from(req.state)
            .map_err(|()| tonic::Status::unknown("failed to convert to vm state"))?;

        // inner_vm 显式 drop，防止提前释放锁
        // 合并 inner_vm 的声明和唯一用途，防止提前 drop
        let (last_accepted_id, block) = {
            let inner_vm = self.vm.write().await;
            inner_vm
                .set_state(state)
                .await
                .map_err(|e| tonic::Status::unknown(e.to_string()))?;
            let last_accepted_id = inner_vm.last_accepted().await?;
            let block = inner_vm
                .get_block(last_accepted_id)
                .await
                .map_err(|e| tonic::Status::unknown(e.to_string()))?;
            drop(inner_vm);
            (last_accepted_id, block)
        };

        Ok(Response::new(vm::SetStateResponse {
            last_accepted_id: Bytes::from(last_accepted_id.to_vec()),
            last_accepted_parent_id: Bytes::from(block.parent().await.to_vec()),
            height: block.height().await,
            bytes: Bytes::from(block.bytes().await.to_vec()),
            timestamp: Some(timestamp_from_time(
                &Utc.timestamp_opt(
                    i64::try_from(block.timestamp().await).unwrap_or_default(),
                    0,
                )
                .unwrap(),
            )),
        }))
    }

    async fn set_preference(
        &self,
        req: Request<vm::SetPreferenceRequest>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("set_preference called");

        let req = req.into_inner();
        self.vm
            .read()
            .await
            .set_preference(ids::Id::from_slice(&req.id))
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn health(
        &self,
        _req: Request<Empty>,
    ) -> std::result::Result<Response<vm::HealthResponse>, tonic::Status> {
        log::debug!("health called");

        let resp = self
            .vm
            .read()
            .await
            .health_check()
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(vm::HealthResponse {
            details: Bytes::from(resp),
        }))
    }

    async fn version(
        &self,
        _req: Request<Empty>,
    ) -> std::result::Result<Response<vm::VersionResponse>, tonic::Status> {
        log::debug!("version called");

        let version = self
            .vm
            .read()
            .await
            .version()
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(vm::VersionResponse { version }))
    }

    async fn connected(
        &self,
        req: Request<vm::ConnectedRequest>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("connected called");

        let req = req.into_inner();
        let node_id = ids::node::Id::from_slice(&req.node_id);
        self.vm
            .read()
            .await
            .connected(&node_id)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn disconnected(
        &self,
        req: Request<vm::DisconnectedRequest>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("disconnected called");

        let req = req.into_inner();
        let node_id = ids::node::Id::from_slice(&req.node_id);
        self.vm
            .read()
            .await
            .disconnected(&node_id)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn app_request(
        &self,
        req: Request<vm::AppRequestMsg>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("app_request called");

        let req = req.into_inner();
        let node_id = ids::node::Id::from_slice(&req.node_id);
        let ts = req.deadline.as_ref().expect("timestamp");
        let deadline = Utc
            .timestamp_opt(
                ts.seconds,
                u32::try_from(ts.nanos).expect("nanos must fit in u32"),
            )
            .single()
            .unwrap();

        self.vm
            .read()
            .await
            .app_request(&node_id, req.request_id, deadline, &req.request)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn app_request_failed(
        &self,
        req: Request<vm::AppRequestFailedMsg>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("app_request_failed called");

        let req = req.into_inner();
        let node_id = ids::node::Id::from_slice(&req.node_id);
        self.vm
            .read()
            .await
            .app_request_failed(&node_id, req.request_id)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn app_response(
        &self,
        req: Request<vm::AppResponseMsg>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("app_response called");

        let req = req.into_inner();
        let node_id = ids::node::Id::from_slice(&req.node_id);
        self.vm
            .read()
            .await
            .app_response(&node_id, req.request_id, &req.response)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn app_gossip(
        &self,
        req: Request<vm::AppGossipMsg>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("app_gossip called");

        let req = req.into_inner();
        let node_id = ids::node::Id::from_slice(&req.node_id);
        self.vm
            .read()
            .await
            .app_gossip(&node_id, &req.msg)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn block_verify(
        &self,
        req: Request<vm::BlockVerifyRequest>,
    ) -> std::result::Result<Response<vm::BlockVerifyResponse>, tonic::Status> {
        log::debug!("block_verify called");

        let req = req.into_inner();
        let mut block = self
            .vm
            .read()
            .await
            .parse_block(&req.bytes)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        block
            .verify()
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(vm::BlockVerifyResponse {
            timestamp: Some(timestamp_from_time(
                &Utc.timestamp_opt(
                    i64::try_from(block.timestamp().await).unwrap_or_default(),
                    0,
                )
                .unwrap(),
            )),
        }))
    }

    async fn block_accept(
        &self,
        req: Request<vm::BlockAcceptRequest>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("block_accept called");

        let req = req.into_inner();
        let id = ids::Id::from_slice(&req.id);

        let mut block = self
            .vm
            .read()
            .await
            .get_block(id)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        block
            .accept()
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }
    async fn block_reject(
        &self,
        req: Request<vm::BlockRejectRequest>,
    ) -> std::result::Result<Response<Empty>, tonic::Status> {
        log::debug!("block_reject called");

        let req = req.into_inner();
        let id = ids::Id::from_slice(&req.id);

        let mut block = self
            .vm
            .read()
            .await
            .get_block(id)
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        block
            .reject()
            .await
            .map_err(|e| tonic::Status::unknown(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn get_ancestors(
        &self,
        req: Request<vm::GetAncestorsRequest>,
    ) -> std::result::Result<Response<vm::GetAncestorsResponse>, tonic::Status> {
        log::debug!("get_ancestors called");
        let req = req.into_inner();

        let block_id = ids::Id::from_slice(req.blk_id.as_ref());
        let _max_blocks_size = usize::try_from(req.max_blocks_size).expect("cast from i32");
        let max_blocks_num = usize::try_from(req.max_blocks_num).expect("cast from i32");
        let max_blocks_retrival_time = Duration::from_secs(
            req.max_blocks_retrival_time
                .try_into()
                .expect("valid timestamp"),
        );

        let ancestors = self
            .vm_ancestors(
                req.blk_id.as_ref(),
                req.max_blocks_num,
                req.max_blocks_size,
                max_blocks_retrival_time,
            )
            .await
            .map(|blks_bytes| Response::new(vm::GetAncestorsResponse { blks_bytes }));

        let e = match ancestors {
            Ok(ancestors) => return Ok(ancestors),
            Err(e) => e,
        };

        if e.kind() != std::io::ErrorKind::Unsupported {
            return Err(tonic::Status::unknown(e.to_string()));
        }

        // not supported by underlying vm use local logic
        let start = Instant::now();
        let block = match self.vm.read().await.get_block(block_id).await {
            Ok(b) => b,
            Err(e) => {
                // special case ErrNotFound as an empty response: this signals
                // the client to avoid contacting this node for further ancestors
                // as they may have been pruned or unavailable due to state-sync.
                return if errors::is_not_found(&e) {
                    log::debug!("get_ancestors local get_block returned: not found");

                    Ok(Response::new(vm::GetAncestorsResponse {
                        blks_bytes: Vec::new(),
                    }))
                } else {
                    Err(e.into())
                };
            }
        };

        let mut ancestors = Vec::with_capacity(max_blocks_num);
        let mut block_opt = Some(block);
        for _ in 0..max_blocks_num {
            let Some(block) = block_opt.take() else { break };

            // 先 clone/copy parent_id，避免 .await 期间 block 被借用
            let parent_id = block.parent().await;

            // 先 clone/copy bytes 数据，确保拥有所有权，彻底规避生命周期问题
            let block_bytes = block.bytes().await;
            ancestors.push(Bytes::copy_from_slice(block_bytes));

            if start.elapsed() > max_blocks_retrival_time {
                log::debug!("get_ancestors exceeded max block retrival time");
                break;
            }
            block_opt = match self.vm.read().await.get_block(parent_id).await {
                Ok(parent) => Some(parent),
                Err(e) => {
                    if errors::is_not_found(&e) {
                        log::debug!("failed to get block during ancestors lookup parentId: {parent_id}: {e}");
                    }
                    None
                }
            };
        }

        Ok(Response::new(vm::GetAncestorsResponse {
            blks_bytes: ancestors,
        }))
    }

    async fn batched_parse_block(
        &self,
        req: Request<vm::BatchedParseBlockRequest>,
    ) -> std::result::Result<Response<vm::BatchedParseBlockResponse>, tonic::Status> {
        log::debug!("batched_parse_block called");
        let req = req.into_inner();

        let to_parse = req
            .request
            .into_iter()
            .map(|bytes| Request::new(vm::ParseBlockRequest { bytes }))
            .map(|request| async {
                self.parse_block(request)
                    .await
                    .map(tonic::Response::into_inner)
            });
        let blocks = futures::future::try_join_all(to_parse).await?;

        Ok(Response::new(vm::BatchedParseBlockResponse {
            response: blocks,
        }))
    }

    #[cfg(not(feature = "subnet_metrics"))]
    async fn gather(
        &self,
        _req: Request<Empty>,
    ) -> std::result::Result<Response<vm::GatherResponse>, tonic::Status> {
        log::debug!("gather called");

        let metric_families =
            vec![crate::proto::pb::io::prometheus::client::MetricFamily::default()];

        Ok(Response::new(vm::GatherResponse { metric_families }))
    }

    #[cfg(feature = "subnet_metrics")]
    #[cfg_attr(docsrs, doc(cfg(feature = "subnet_metrics")))]
    async fn gather(
        &self,
        _req: Request<Empty>,
    ) -> std::result::Result<Response<vm::GatherResponse>, tonic::Status> {
        log::debug!("gather called");

        // ref. <https://prometheus.io/docs/instrumenting/writing_clientlibs/#process-metrics>
        let metric_families = crate::subnet::rpc::metrics::MetricsFamilies::from(
            &self.process_metrics.read().await.gather(),
        )
        .mfs;

        Ok(Response::new(vm::GatherResponse { metric_families }))
    }

    async fn state_sync_enabled(
        &self,
        _req: Request<Empty>,
    ) -> std::result::Result<Response<vm::StateSyncEnabledResponse>, tonic::Status> {
        log::debug!("state_sync_enabled called");

        // TODO: Implement state sync request/response
        Ok(Response::new(vm::StateSyncEnabledResponse {
            enabled: false,
            err: 0,
        }))
    }

    async fn get_ongoing_sync_state_summary(
        &self,
        _req: Request<Empty>,
    ) -> std::result::Result<Response<vm::GetOngoingSyncStateSummaryResponse>, tonic::Status> {
        log::debug!("get_ongoing_sync_state_summary called");

        Err(tonic::Status::unimplemented(
            "get_ongoing_sync_state_summary",
        ))
    }

    async fn parse_state_summary(
        &self,
        _req: Request<vm::ParseStateSummaryRequest>,
    ) -> std::result::Result<tonic::Response<vm::ParseStateSummaryResponse>, tonic::Status> {
        log::debug!("parse_state_summary called");

        Err(tonic::Status::unimplemented("parse_state_summary"))
    }

    async fn get_state_summary(
        &self,
        _req: Request<vm::GetStateSummaryRequest>,
    ) -> std::result::Result<Response<vm::GetStateSummaryResponse>, tonic::Status> {
        log::debug!("get_state_summary called");

        Err(tonic::Status::unimplemented("get_state_summary"))
    }

    async fn get_last_state_summary(
        &self,
        _req: Request<Empty>,
    ) -> std::result::Result<Response<vm::GetLastStateSummaryResponse>, tonic::Status> {
        log::debug!("get_last_state_summary called");

        Err(tonic::Status::unimplemented("get_last_state_summary"))
    }

    async fn state_summary_accept(
        &self,
        _req: Request<vm::StateSummaryAcceptRequest>,
    ) -> std::result::Result<tonic::Response<vm::StateSummaryAcceptResponse>, tonic::Status> {
        log::debug!("state_summary_accept called");

        Err(tonic::Status::unimplemented("state_summary_accept"))
    }

    async fn get_block_id_at_height(
        &self,
        req: Request<vm::GetBlockIdAtHeightRequest>,
    ) -> std::result::Result<Response<vm::GetBlockIdAtHeightResponse>, tonic::Status> {
        log::debug!("get_block_id_at_height called");

        let msg = req.into_inner();
        let inner_vm = self.vm.read().await;

        match inner_vm.get_block_id_at_height(msg.height).await {
            Ok(height) => {
                return Ok(Response::new(vm::GetBlockIdAtHeightResponse {
                    blk_id: height.to_vec().into(),
                    err: 0,
                }))
            }
            Err(e) => {
                if error_to_error_code(&e.to_string()) != 0 {
                    return Ok(Response::new(vm::GetBlockIdAtHeightResponse {
                        blk_id: vec![].into(),
                        err: error_to_error_code(&e.to_string()),
                    }));
                }
                return Err(tonic::Status::unknown(e.to_string()));
            }
        }
    }
}
