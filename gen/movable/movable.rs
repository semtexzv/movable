#![allow(unused_imports)]
#![allow(nonstandard_style)]
#![allow(unreachable_patterns)]
#![allow(clippy::module_inception)]
use protokit::*;
pub fn register_types(registry: &mut protokit::textformat::reflect::Registry) {
    registry.register(&Chunk::default());
    registry.register(&Params::default());
    registry.register(&File::default());
    registry.register(&Meta::default());
    registry.register(&Info::default());
    registry.register(&Create::default());
    registry.register(&Delete::default());
    registry.register(&Copy::default());
    registry.register(&Data::default());
    registry.register(&Update::default());
}
#[derive(Debug, Default, Clone, PartialEq, Proto)]
#[proto(name = "Chunk", package = "movable")]
pub struct Chunk {
    #[field(1u32, "hash", fixed64, singular)]
    pub hash: u64,
    #[field(2u32, "start", varint, singular)]
    pub start: u64,
    #[field(3u32, "len", varint, singular)]
    pub len: u32,
}
#[derive(Debug, Default, Clone, PartialEq, Proto)]
#[proto(name = "Params", package = "movable")]
pub struct Params {
    #[field(1u32, "min_size", varint, singular)]
    pub min_size: u32,
    #[field(2u32, "avg_size", varint, singular)]
    pub avg_size: u32,
    #[field(3u32, "max_size", varint, singular)]
    pub max_size: u32,
}
#[derive(Debug, Default, Clone, PartialEq, Proto)]
#[proto(name = "File", package = "movable")]
pub struct File {
    #[field(1u32, "path", string, singular)]
    pub path: String,
    #[field(2u32, "params", nested, optional)]
    pub params: Option<Box<Params>>,
    #[field(3u32, "chunks", nested, repeated)]
    pub chunks: Vec<Chunk>,
}
#[derive(Debug, Default, Clone, PartialEq, Proto)]
#[proto(name = "Meta", package = "movable")]
pub struct Meta {
    #[field(1u32, "volume", string, singular)]
    pub volume: String,
}
#[derive(Debug, Clone, PartialEq, Proto)]
pub enum InfoOneOfKind {
    #[field(1u32, "meta", nested, raw)]
    Meta(Meta),
    #[field(2u32, "file", nested, raw)]
    File(File),
    #[field(3u32, "done", bool, raw)]
    Done(bool),
    __Unused(::core::marker::PhantomData<&'static ()>),
}
impl Default for InfoOneOfKind {
    fn default() -> Self {
        Self::Meta(Default::default())
    }
}
#[derive(Debug, Default, Clone, PartialEq, Proto)]
#[proto(name = "Info", package = "movable")]
pub struct Info {
    #[oneof([1u32, 2u32, 3u32], ["meta", "file", "done"])]
    pub Kind: Option<InfoOneOfKind>,
}
#[derive(Debug, Default, Clone, PartialEq, Proto)]
#[proto(name = "Create", package = "movable")]
pub struct Create {
    #[field(1u32, "path", string, singular)]
    pub path: String,
}
#[derive(Debug, Default, Clone, PartialEq, Proto)]
#[proto(name = "Delete", package = "movable")]
pub struct Delete {
    #[field(1u32, "path", string, singular)]
    pub path: String,
}
#[derive(Debug, Default, Clone, PartialEq, Proto)]
#[proto(name = "Copy", package = "movable")]
pub struct Copy {
    #[field(1u32, "src_path", string, singular)]
    pub src_path: String,
    #[field(2u32, "dst_path", string, singular)]
    pub dst_path: String,
}
#[derive(Debug, Default, Clone, PartialEq, Proto)]
#[proto(name = "Data", package = "movable")]
pub struct Data {
    #[field(1u32, "data", bytes, singular)]
    pub data: Vec<u8>,
    #[field(2u32, "pos", varint, singular)]
    pub pos: u64,
    #[field(3u32, "len", varint, singular)]
    pub len: u64,
}
#[derive(Debug, Clone, PartialEq, Proto)]
pub enum UpdateOneOfKind {
    #[field(1u32, "create", nested, raw)]
    Create(Create),
    #[field(2u32, "delete", nested, raw)]
    Delete(Delete),
    #[field(3u32, "copy", nested, raw)]
    Copy(Copy),
    #[field(4u32, "data", nested, raw)]
    Data(Data),
    #[field(5u32, "done", bool, raw)]
    Done(bool),
    __Unused(::core::marker::PhantomData<&'static ()>),
}
impl Default for UpdateOneOfKind {
    fn default() -> Self {
        Self::Create(Default::default())
    }
}
#[derive(Debug, Default, Clone, PartialEq, Proto)]
#[proto(name = "Update", package = "movable")]
pub struct Update {
    #[oneof(
        [1u32,
        2u32,
        3u32,
        4u32,
        5u32,
        ],
        ["create",
        "delete",
        "copy",
        "data",
        "done",
        ]
    )]
    pub Kind: Option<UpdateOneOfKind>,
}
mod Movable_server {
    use super::*;
    use protokit::grpc::*;
    #[protokit::grpc::async_trait]
    pub trait Movable: Send + Sync + 'static {
        type SyncStream: Stream<Item = Result<super::Update, Status>> + Send + 'static;
        async fn sync(
            &self,
            req: tonic::Request<tonic::Streaming<super::Info>>,
        ) -> Result<tonic::Response<Self::SyncStream>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct MovableServer<S: Movable>(pub Arc<S>);
    impl<S: Movable> Clone for MovableServer<S> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<S: Movable> From<S> for MovableServer<S> {
        fn from(v: S) -> Self {
            Self(::std::sync::Arc::new(v))
        }
    }
    impl<S: Movable> From<::std::sync::Arc<S>> for MovableServer<S> {
        fn from(v: ::std::sync::Arc<S>) -> Self {
            Self(v)
        }
    }
    struct SyncSvc<S: Movable>(Arc<S>);
    impl<S: Movable> tonic::server::StreamingService<super::Info> for SyncSvc<S> {
        type Response = super::Update;
        type ResponseStream = S::SyncStream;
        type Future = BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
        fn call(
            &mut self,
            request: tonic::Request<tonic::Streaming<super::Info>>,
        ) -> Self::Future {
            let inner = self.0.clone();
            Box::pin(async move { inner.sync(request).await })
        }
    }
    impl<S, B> Service<http::Request<B>> for MovableServer<S>
    where
        S: Movable,
        B: Body + Send + 'static,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>> + Send
            + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = core::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            match req.uri().path() {
                "/movable.Movable/Sync" => {
                    let inner = self.0.clone();
                    let fut = async move {
                        let method = SyncSvc(inner);
                        let codec = protokit::grpc::TonicCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec);
                        let res = grpc.streaming(method, req).await;
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
    impl<S: Movable> tonic::transport::NamedService for MovableServer<S> {
        const NAME: &'static str = "movable.Movable";
    }
}
pub use Movable_server::*;
mod Movable_client {
    use super::*;
    use protokit::grpc::*;
    #[derive(Debug, Clone)]
    pub struct MovableClient<C> {
        inner: tonic::client::Grpc<C>,
    }
    impl MovableClient<tonic::transport::Channel> {
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: core::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<S> MovableClient<S>
    where
        S: tonic::client::GrpcService<tonic::body::BoxBody>,
        S::Error: Into<StdError>,
        S::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <S::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: S) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: S,
            interceptor: F,
        ) -> MovableClient<InterceptedService<S, F>>
        where
            F: tonic::service::Interceptor,
            S::ResponseBody: Default,
            S: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <S as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <S as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            MovableClient::new(InterceptedService::new(inner, interceptor))
        }
        pub async fn sync(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::Info>,
        ) -> Result<tonic::Response<tonic::Streaming<super::Update>>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    Status::new(
                        Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = protokit::grpc::TonicCodec::default();
            let path = http::uri::PathAndQuery::from_static("/movable.Movable/Sync");
            self.inner.streaming(request.into_streaming_request(), path, codec).await
        }
    }
}
pub use Movable_client::*;
