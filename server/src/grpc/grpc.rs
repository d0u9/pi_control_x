use ::std::future::Future;
use ::std::net::SocketAddr;
use ::tonic::transport::Server;
use crate::result::{Result, Error};

use super::api_server::DiskApiServer;

#[derive(Debug, Default)]
pub struct Builder {
    addr: Option<SocketAddr>,
}

impl Builder {
    pub fn new() -> Self {
        Builder::default()
    }

    pub fn address(mut self, addr: &str) -> Result<Self> {
        let addr = addr.parse()?;
        self.addr = Some(addr);
        Ok(self)
    }

    pub fn commit(self) -> Result<GrpcServer> {
        self.addr.ok_or(Error::with_str("no address assigned"))?;
        let disk_service = DiskApiServer::new();

        let grpc_server = GrpcServer {
            addr: self.addr,
            disk_service,
        };

        Ok(grpc_server)
    }
}

pub struct GrpcServer {
    addr: Option<SocketAddr>,
    disk_service: DiskApiServer,
}

impl GrpcServer {
    pub async fn serve(self, shutdown: impl Future<Output = ()>) -> Result<()> {
        let disk_service = self.disk_service.service();
        let addr = self.addr.unwrap();

        println!("GRPC server is listening on: {}", &self.addr.unwrap());

        Server::builder()
            .add_service(disk_service)
            .serve(addr)
            .await
            .unwrap();

        Ok(())
    }
}
