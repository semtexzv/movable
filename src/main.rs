use std::collections::BTreeMap;
use std::io;
use std::io::ErrorKind;
use std::os::unix::prelude::MetadataExt;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use async_fn_stream::try_fn_stream;
use bollard::Docker;
use docker_plugin::volume::*;
use hyperlocal::UnixServerExt;
use protokit::grpc::futures::StreamExt;
use tonic::Status;
use futures::Stream;
use futures::stream::BoxStream;
use crate::svc::movable::movable::{Delete, File, Info, InfoOneOfKind, UpdateOneOfKind};

pub const MEGABYTE: u64 = 1024 * 1024;
pub const GIGABYTE: u64 = 1024 * 1024 * 1024;

pub fn determine_params(size: u64) -> (u32, u32, u32) {
    match size {
        0..=MEGABYTE => (32 * 1024, 128 * 1024, 512 * 1024),
        MEGABYTE..=GIGABYTE => (512 * 1024, 2 * 1024 * 1024, 64 * 2014 * 1024),
        GIGABYTE.. => (fastcdc::v2020::MINIMUM_MAX, fastcdc::v2020::AVERAGE_MAX, fastcdc::v2020::MAXIMUM_MAX)
    }
}

mod svc {
    include!("../gen/mod.rs");
}

pub struct Movable {
    root: PathBuf,
    docker: Docker,
}

#[async_trait::async_trait]
impl docker_plugin::volume::Driver for Movable {
    type Opts = ();
    type Status = ();

    async fn create(&self, req: CreateRequest<Self::Opts>) -> anyhow::Result<()> {
        todo!()
    }

    async fn list(&self) -> anyhow::Result<ListResponse<Self::Status>> {
        todo!()
    }

    async fn get(&self, req: GetRequest) -> anyhow::Result<GetResponse<Self::Status>> {
        todo!()
    }

    async fn remove(&self, req: RemoveRequest) -> anyhow::Result<()> {
        todo!()
    }

    async fn path(&self, req: PathRequest) -> anyhow::Result<PathResponse> {
        todo!()
    }

    async fn mount(&self, req: MountRequest) -> anyhow::Result<MountResponse> {
        todo!()
    }

    async fn unmount(&self, req: UnmountRequest) -> anyhow::Result<()> {
        todo!()
    }

    async fn capabilities(&self) -> CapabilitiesResponse {
        todo!()
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
        let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}

#[async_trait::async_trait]
impl svc::movable::movable::Movable for Movable {
    type SyncStream = BoxStream<'static, Result<svc::movable::movable::Update, protokit::grpc::Status>>;
    async fn sync(&self, req: tonic::Request<tonic::Streaming<svc::movable::movable::Info>>) -> Result<tonic::Response<Self::SyncStream>, protokit::grpc::Status> {
        let mut inner = req.into_inner();

        let mut root = None;
        let mut files = BTreeMap::new();
        let mut ok = false;

        while let Some(msg) = inner.next().await {
            let msg = msg.expect("Failed to deserialize msg");
            match msg.Kind {
                Some(InfoOneOfKind::Meta(meta)) => {
                    root = Some(meta.volume)
                }
                Some(InfoOneOfKind::File(file)) => {
                    files.insert(PathBuf::from(&file.path), file);
                }
                Some(InfoOneOfKind::Done(done)) => {
                    ok = true;
                    break;
                }
                other => panic!("{other:?} was provided"),
            }
        }

        if !ok {
            panic!("Dropped");
        }

        let root = self.root.clone();

        Ok(tonic::Response::new(Box::pin(try_fn_stream(|sink| async move {
            for (path, file) in files {
                let params = file.params.expect("Missing file params");
                let fpath = root.join(&path);
                match std::fs::metadata(&fpath) {
                    Err(e) if e.kind() == ErrorKind::NotFound => {
                        sink.emit(svc::movable::movable::Update {
                            Kind: Some(UpdateOneOfKind::Delete(Delete {
                                path: path.to_string_lossy().to_string(),
                            })),
                        }).await
                    }
                    Err(e) => panic!("{:?}", e),
                    Ok(v) => {
                        let local = std::fs::File::open(&fpath)?;
                        let local = unsafe { memmap::Mmap::map(&local).expect("Memmap failed") };
                        let cdc = fastcdc::v2020::FastCDC::new(&local, params.min_size, params.avg_size, params.max_size);
                        let chunks = cdc.collect::<Vec<_>>();
                        // if file.chunks
                    }
                }
            }

            Ok(())
        }))))
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let docker = Docker::connect_with_unix_defaults().unwrap();

    let driver = Movable {
        root: Path::new(".").to_path_buf(),
        docker,
    };

    let driver = Arc::new(driver);

    let app = docker_plugin::router(vec![
        docker_plugin::volume::IMPLEMENTS_VOLUME.to_string()
    ]);

    let app = app.merge(docker_plugin::volume::router(driver));

    let _ = std::fs::remove_file("/run/docker/plugins/movable.sock");

    axum::Server::bind_unix("/run/docker/plugins/movable.sock")
        .expect("Couldn't bind unix socket")
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Can't start the server");

    let _ = std::fs::remove_file("/run/docker/plugins/movable.sock");
}
