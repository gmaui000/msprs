use tonic::{Request, Response, Status};

use crate::utils::config::Config;

use crate::gss::{
    gbt_stream_service_server::GbtStreamService, BindStreamPortRequest, BindStreamPortResponse,
    FreeStreamPortRequest, FreeStreamPortResponse,
};

pub struct StreamTask {
    pub cancel_tx: tokio::sync::broadcast::Sender<()>,
    pub udp_join_handle: tokio::task::JoinHandle<()>,
    pub tcp_join_handle: tokio::task::JoinHandle<()>,
}

pub struct MyGbtStreamService {
    pub config: Config,
    ports: std::sync::Mutex<std::collections::LinkedList<u16>>,
    pub join_handlers: std::sync::Mutex<std::collections::HashMap<u16, StreamTask>>,
}

impl MyGbtStreamService {
    pub fn new(config: Config) -> Self {
        let start = config.stream_port_start;
        let stop = config.stream_port_stop;
        MyGbtStreamService {
            config,
            ports: (start..=stop)
                .collect::<std::collections::LinkedList<u16>>()
                .into(),
            join_handlers: std::collections::HashMap::<u16, StreamTask>::new().into(),
        }
    }

    pub fn pop_port(&self) -> u16 {
        match self.ports.lock().unwrap().pop_front() {
            None => {
                tracing::error!("No ports are free");
                0
            }
            Some(port) => port,
        }
    }

    pub fn push_port(&self, port: u16) {
        self.ports.lock().unwrap().push_back(port);
    }

    pub fn push_task(
        &self,
        port: u16,
        cancel_tx: tokio::sync::broadcast::Sender<()>,
        udp_join_handle: tokio::task::JoinHandle<()>,
        tcp_join_handle: tokio::task::JoinHandle<()>,
    ) {
        if let Ok(mut join_handlers) = self.join_handlers.lock() {
            join_handlers.insert(
                port,
                StreamTask {
                    cancel_tx,
                    udp_join_handle,
                    tcp_join_handle,
                },
            );
        }
    }

    pub async fn pop_task(&self, port: u16) {
        let mut udp_handle: Option<tokio::task::JoinHandle<()>> = None;
        let mut tcp_handle: Option<tokio::task::JoinHandle<()>> = None;

        if let Ok(mut join_handlers) = self.join_handlers.lock() {
            if let Some(task) = join_handlers.remove(&port) {
                let _ = task.cancel_tx.send(());
                udp_handle = Some(task.udp_join_handle);
                tcp_handle = Some(task.tcp_join_handle);
            }
        }

        if let (Some(u), Some(t)) = (udp_handle, tcp_handle) {
            let _ = tokio::join!(u, t);
        }
    }
}

#[tonic::async_trait]
impl GbtStreamService for MyGbtStreamService {
    async fn bind_stream_port(
        &self,
        request: Request<BindStreamPortRequest>,
    ) -> Result<Response<BindStreamPortResponse>, Status> {
        self.rpc_bind_stream_port(request).await
    }

    async fn free_stream_port(
        &self,
        request: Request<FreeStreamPortRequest>,
    ) -> Result<Response<FreeStreamPortResponse>, Status> {
        self.rpc_free_stream_port(request).await
    }
}
