/// Mapping of build_info::Version
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BidderVersionInfo {
    #[prost(string, tag = "1")]
    pub git_commit: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub git_ref: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub build_time_utc: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Empty {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MustWinBlockParams {
    #[prost(uint64, tag = "1")]
    pub block: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateNewBidParams {
    #[prost(double, tag = "1")]
    pub seen_time: f64,
    #[prost(string, tag = "2")]
    pub publisher_name: ::prost::alloc::string::String,
    #[prost(enumeration = "PublisherType", tag = "3")]
    pub publisher_type: i32,
    #[prost(double, optional, tag = "4")]
    pub relay_time: ::core::option::Option<f64>,
    #[prost(string, tag = "5")]
    pub relay_name: ::prost::alloc::string::String,
    /// Array of 32 bytes
    #[prost(bytes = "vec", tag = "6")]
    pub block_hash: ::prost::alloc::vec::Vec<u8>,
    /// Array of 32 bytes
    #[prost(bytes = "vec", tag = "7")]
    pub parent_hash: ::prost::alloc::vec::Vec<u8>,
    /// Array of 4 uint64
    #[prost(uint64, repeated, tag = "8")]
    pub value: ::prost::alloc::vec::Vec<u64>,
    #[prost(uint64, tag = "9")]
    pub slot_number: u64,
    #[prost(uint64, tag = "10")]
    pub block_number: u64,
    /// Array of 0 or 48 bytes
    #[prost(bytes = "vec", tag = "11")]
    pub builder_pubkey: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, optional, tag = "12")]
    pub extra_data: ::core::option::Option<::prost::alloc::string::String>,
    /// Array of 0 or 20 bytes
    #[prost(bytes = "vec", tag = "13")]
    pub fee_recipient: ::prost::alloc::vec::Vec<u8>,
    /// Array of 0 or 20 bytes
    #[prost(bytes = "vec", tag = "14")]
    pub proposer_fee_recipient: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, optional, tag = "15")]
    pub gas_used: ::core::option::Option<u64>,
    #[prost(bool, optional, tag = "16")]
    pub optimistic_submission: ::core::option::Option<bool>,
    /// For metrics
    #[prost(uint64, tag = "17")]
    pub creation_time_us: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewBlockParams {
    #[prost(uint64, tag = "1")]
    pub session_id: u64,
    /// Array of 4 uint64
    #[prost(uint64, repeated, tag = "2")]
    pub true_block_value: ::prost::alloc::vec::Vec<u64>,
    #[prost(bool, tag = "3")]
    pub can_add_payout_tx: bool,
    #[prost(uint64, tag = "4")]
    pub block_id: u64,
    /// For metrics
    #[prost(uint64, tag = "5")]
    pub creation_time_us: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DestroySlotBidderParams {
    #[prost(uint64, tag = "1")]
    pub session_id: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateSlotBidderParams {
    #[prost(uint64, tag = "1")]
    pub block: u64,
    #[prost(uint64, tag = "2")]
    pub slot: u64,
    /// Array of 32 bytes
    #[prost(bytes = "vec", tag = "3")]
    pub parent_hash: ::prost::alloc::vec::Vec<u8>,
    /// Id identifying the session. Used in all following calls.
    #[prost(uint64, tag = "4")]
    pub session_id: u64,
    /// unix ts
    #[prost(int64, tag = "5")]
    pub slot_timestamp: i64,
}
/// Info about a onchain block from reth.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LandedBlockInfo {
    #[prost(uint64, tag = "1")]
    pub block_number: u64,
    #[prost(int64, tag = "2")]
    pub block_timestamp: i64,
    /// Array of 4 uint64
    #[prost(uint64, repeated, tag = "3")]
    pub builder_balance: ::prost::alloc::vec::Vec<u64>,
    /// true -> we landed this block.
    /// If false we could have landed it in coinbase == fee recipient mode but balance wouldn't change so we don't care.
    #[prost(bool, tag = "4")]
    pub beneficiary_is_builder: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LandedBlocksParams {
    /// Added field name
    #[prost(message, repeated, tag = "1")]
    pub landed_block_info: ::prost::alloc::vec::Vec<LandedBlockInfo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Bid {
    /// Optional implicitly by allowing empty
    ///
    /// Array of 4 uint64
    #[prost(uint64, repeated, tag = "1")]
    pub payout_tx_value: ::prost::alloc::vec::Vec<u64>,
    #[prost(uint64, tag = "2")]
    pub block_id: u64,
    /// Optional implicitly by allowing empty
    ///
    /// Array of 4 uint64
    #[prost(uint64, repeated, tag = "3")]
    pub seen_competition_bid: ::prost::alloc::vec::Vec<u64>,
    #[prost(uint64, optional, tag = "4")]
    pub trigger_creation_time_us: ::core::option::Option<u64>,
}
/// Exactly 1 member will be not null.
/// Since this is not mapped to an enum we must be careful to manually update BiddingServiceClientAdapter.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Callback {
    #[prost(message, optional, tag = "1")]
    pub bid: ::core::option::Option<Bid>,
    #[prost(bool, optional, tag = "2")]
    pub can_use_suggested_fee_recipient_as_coinbase_change: ::core::option::Option<bool>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum PublisherType {
    RelayBids = 0,
    RelayHeaders = 1,
    UltrasoundWs = 2,
    BloxrouteWs = 3,
}
impl PublisherType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            PublisherType::RelayBids => "RelayBids",
            PublisherType::RelayHeaders => "RelayHeaders",
            PublisherType::UltrasoundWs => "UltrasoundWs",
            PublisherType::BloxrouteWs => "BloxrouteWs",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "RelayBids" => Some(Self::RelayBids),
            "RelayHeaders" => Some(Self::RelayHeaders),
            "UltrasoundWs" => Some(Self::UltrasoundWs),
            "BloxrouteWs" => Some(Self::BloxrouteWs),
            _ => None,
        }
    }
}
/// Generated client implementations.
pub mod bidding_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Protocol for the bidding service. It's used to marshal all the traits in src/block_descriptor_bidding/traits.rs
    /// Usage:
    /// The client connects to the server and calls Initialize, this call should create the real BiddingService on the server side.
    /// Before calling Initialize any other call will fail. Initialize can be called again to recreate the BiddingService (eg: rbuilder reconnection).
    /// After that, for each slot the client should call CreateSlotBidder to create the SlotBidder on the server side and DestroySlotBidder when the SlotBidder is not needed anymore.
    /// Other calls are almost 1 to 1 with the original traits but for SlotBidder calls block/slot are added to identify the SlotBidder.
    /// Notice that CreateSlotBidder returns a stream of Callback. This stream is used for 2 things:
    /// - Send back bids made by the SlotBidder.
    /// - Notify changes on the state of SlotBidder's can_use_suggested_fee_recipient_as_coinbase. We use this methodology instead of a
    ///   forward RPC call since can_use_suggested_fee_recipient_as_coinbase almost does not change and we want to avoid innecesary RPC calls during block building.
    #[derive(Debug, Clone)]
    pub struct BiddingServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl BiddingServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> BiddingServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> BiddingServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            BiddingServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Call after connection before calling anything. This will really create the BiddingService on the server side.
        /// Returns the version info for the server side.
        pub async fn initialize(
            &mut self,
            request: impl tonic::IntoRequest<super::LandedBlocksParams>,
        ) -> Result<tonic::Response<super::BidderVersionInfo>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/bidding_service.BiddingService/Initialize",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// BiddingService
        pub async fn create_slot_bidder(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateSlotBidderParams>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::Callback>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/bidding_service.BiddingService/CreateSlotBidder",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        pub async fn destroy_slot_bidder(
            &mut self,
            request: impl tonic::IntoRequest<super::DestroySlotBidderParams>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/bidding_service.BiddingService/DestroySlotBidder",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn must_win_block(
            &mut self,
            request: impl tonic::IntoRequest<super::MustWinBlockParams>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/bidding_service.BiddingService/MustWinBlock",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn update_new_landed_blocks_detected(
            &mut self,
            request: impl tonic::IntoRequest<super::LandedBlocksParams>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/bidding_service.BiddingService/UpdateNewLandedBlocksDetected",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn update_failed_reading_new_landed_blocks(
            &mut self,
            request: impl tonic::IntoRequest<super::Empty>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/bidding_service.BiddingService/UpdateFailedReadingNewLandedBlocks",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// BiddingService->BlockBidWithStatsObs
        pub async fn update_new_bid(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateNewBidParams>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/bidding_service.BiddingService/UpdateNewBid",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// UnfinishedBlockBuildingSink
        pub async fn new_block(
            &mut self,
            request: impl tonic::IntoRequest<super::NewBlockParams>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/bidding_service.BiddingService/NewBlock",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod bidding_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with BiddingServiceServer.
    #[async_trait]
    pub trait BiddingService: Send + Sync + 'static {
        /// Call after connection before calling anything. This will really create the BiddingService on the server side.
        /// Returns the version info for the server side.
        async fn initialize(
            &self,
            request: tonic::Request<super::LandedBlocksParams>,
        ) -> Result<tonic::Response<super::BidderVersionInfo>, tonic::Status>;
        /// Server streaming response type for the CreateSlotBidder method.
        type CreateSlotBidderStream: futures_core::Stream<
                Item = Result<super::Callback, tonic::Status>,
            >
            + Send
            + 'static;
        /// BiddingService
        async fn create_slot_bidder(
            &self,
            request: tonic::Request<super::CreateSlotBidderParams>,
        ) -> Result<tonic::Response<Self::CreateSlotBidderStream>, tonic::Status>;
        async fn destroy_slot_bidder(
            &self,
            request: tonic::Request<super::DestroySlotBidderParams>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
        async fn must_win_block(
            &self,
            request: tonic::Request<super::MustWinBlockParams>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
        async fn update_new_landed_blocks_detected(
            &self,
            request: tonic::Request<super::LandedBlocksParams>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
        async fn update_failed_reading_new_landed_blocks(
            &self,
            request: tonic::Request<super::Empty>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
        /// BiddingService->BlockBidWithStatsObs
        async fn update_new_bid(
            &self,
            request: tonic::Request<super::UpdateNewBidParams>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
        /// UnfinishedBlockBuildingSink
        async fn new_block(
            &self,
            request: tonic::Request<super::NewBlockParams>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
    }
    /// Protocol for the bidding service. It's used to marshal all the traits in src/block_descriptor_bidding/traits.rs
    /// Usage:
    /// The client connects to the server and calls Initialize, this call should create the real BiddingService on the server side.
    /// Before calling Initialize any other call will fail. Initialize can be called again to recreate the BiddingService (eg: rbuilder reconnection).
    /// After that, for each slot the client should call CreateSlotBidder to create the SlotBidder on the server side and DestroySlotBidder when the SlotBidder is not needed anymore.
    /// Other calls are almost 1 to 1 with the original traits but for SlotBidder calls block/slot are added to identify the SlotBidder.
    /// Notice that CreateSlotBidder returns a stream of Callback. This stream is used for 2 things:
    /// - Send back bids made by the SlotBidder.
    /// - Notify changes on the state of SlotBidder's can_use_suggested_fee_recipient_as_coinbase. We use this methodology instead of a
    ///   forward RPC call since can_use_suggested_fee_recipient_as_coinbase almost does not change and we want to avoid innecesary RPC calls during block building.
    #[derive(Debug)]
    pub struct BiddingServiceServer<T: BiddingService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: BiddingService> BiddingServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for BiddingServiceServer<T>
    where
        T: BiddingService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/bidding_service.BiddingService/Initialize" => {
                    #[allow(non_camel_case_types)]
                    struct InitializeSvc<T: BiddingService>(pub Arc<T>);
                    impl<
                        T: BiddingService,
                    > tonic::server::UnaryService<super::LandedBlocksParams>
                    for InitializeSvc<T> {
                        type Response = super::BidderVersionInfo;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::LandedBlocksParams>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).initialize(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = InitializeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/bidding_service.BiddingService/CreateSlotBidder" => {
                    #[allow(non_camel_case_types)]
                    struct CreateSlotBidderSvc<T: BiddingService>(pub Arc<T>);
                    impl<
                        T: BiddingService,
                    > tonic::server::ServerStreamingService<
                        super::CreateSlotBidderParams,
                    > for CreateSlotBidderSvc<T> {
                        type Response = super::Callback;
                        type ResponseStream = T::CreateSlotBidderStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateSlotBidderParams>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).create_slot_bidder(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateSlotBidderSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/bidding_service.BiddingService/DestroySlotBidder" => {
                    #[allow(non_camel_case_types)]
                    struct DestroySlotBidderSvc<T: BiddingService>(pub Arc<T>);
                    impl<
                        T: BiddingService,
                    > tonic::server::UnaryService<super::DestroySlotBidderParams>
                    for DestroySlotBidderSvc<T> {
                        type Response = super::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DestroySlotBidderParams>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).destroy_slot_bidder(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DestroySlotBidderSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/bidding_service.BiddingService/MustWinBlock" => {
                    #[allow(non_camel_case_types)]
                    struct MustWinBlockSvc<T: BiddingService>(pub Arc<T>);
                    impl<
                        T: BiddingService,
                    > tonic::server::UnaryService<super::MustWinBlockParams>
                    for MustWinBlockSvc<T> {
                        type Response = super::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MustWinBlockParams>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).must_win_block(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = MustWinBlockSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/bidding_service.BiddingService/UpdateNewLandedBlocksDetected" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateNewLandedBlocksDetectedSvc<T: BiddingService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: BiddingService,
                    > tonic::server::UnaryService<super::LandedBlocksParams>
                    for UpdateNewLandedBlocksDetectedSvc<T> {
                        type Response = super::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::LandedBlocksParams>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).update_new_landed_blocks_detected(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateNewLandedBlocksDetectedSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/bidding_service.BiddingService/UpdateFailedReadingNewLandedBlocks" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateFailedReadingNewLandedBlocksSvc<T: BiddingService>(
                        pub Arc<T>,
                    );
                    impl<T: BiddingService> tonic::server::UnaryService<super::Empty>
                    for UpdateFailedReadingNewLandedBlocksSvc<T> {
                        type Response = super::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Empty>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner)
                                    .update_failed_reading_new_landed_blocks(request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateFailedReadingNewLandedBlocksSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/bidding_service.BiddingService/UpdateNewBid" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateNewBidSvc<T: BiddingService>(pub Arc<T>);
                    impl<
                        T: BiddingService,
                    > tonic::server::UnaryService<super::UpdateNewBidParams>
                    for UpdateNewBidSvc<T> {
                        type Response = super::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateNewBidParams>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).update_new_bid(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateNewBidSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/bidding_service.BiddingService/NewBlock" => {
                    #[allow(non_camel_case_types)]
                    struct NewBlockSvc<T: BiddingService>(pub Arc<T>);
                    impl<
                        T: BiddingService,
                    > tonic::server::UnaryService<super::NewBlockParams>
                    for NewBlockSvc<T> {
                        type Response = super::Empty;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::NewBlockParams>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).new_block(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = NewBlockSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: BiddingService> Clone for BiddingServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: BiddingService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: BiddingService> tonic::server::NamedService for BiddingServiceServer<T> {
        const NAME: &'static str = "bidding_service.BiddingService";
    }
}
