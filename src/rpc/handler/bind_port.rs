use tokio;
use tonic::{Request, Response, Status};

use crate::gss::{BindStreamPortRequest, BindStreamPortResponse, ResponseCode};
use crate::rpc::server::MyGbtStreamService;
use crate::stream;

impl MyGbtStreamService {
    pub async fn rpc_bind_stream_port(
        &self,
        _request: Request<BindStreamPortRequest>,
    ) -> Result<Response<BindStreamPortResponse>, Status> {
        let mut reply = BindStreamPortResponse::default();

        // stub for mediaserver
        if true {
            reply.code = ResponseCode::Ok.into();
            reply.message = String::new();
            reply.media_server_ip = String::from("192.168.31.164");
            reply.media_server_port = 10000;
            return Ok(Response::new(reply));
        }

        // alloc port
        let port = self.pop_port();
        if port == 0 {
            reply.code = ResponseCode::NoPortsFree.into();
            reply.message = ResponseCode::NoPortsFree.as_str_name().to_string();
            return Ok(Response::new(reply));
        }

        // bind
        match stream::server::bind(&self.config.host, port).await {
            Err(e) => {
                tracing::error!("stream::server::bind error, e: {:?}", &e);
                reply.code = ResponseCode::BindPortError.into();
                reply.message = e.to_string();
                Ok(Response::new(reply))
            }
            Ok((stream_udp_socket, stream_tcp_listener)) => {
                // serve
                let stream_handler = stream::handler::StreamHandler::new(
                    self.config.my_ip.clone(),
                    port,
                    stream_udp_socket,
                    stream_tcp_listener,
                );

                let (udp_tcp_cancel_tx, _) = tokio::sync::broadcast::channel(1);
                let arc_stream_handler: std::sync::Arc<stream::handler::StreamHandler> =
                    std::sync::Arc::new(stream_handler);
                match stream::server::run_forever(
                    udp_tcp_cancel_tx.clone(),
                    self.config.socket_recv_buffer_size,
                    arc_stream_handler,
                )
                .await
                {
                    Err(e) => {
                        tracing::error!("stream::server::run_forever error, e: {:?}", &e);
                        reply.code = ResponseCode::RunStreamServiceError.into();
                        reply.message = e.to_string();
                        Ok(Response::new(reply))
                    }
                    Ok((udp_join_handle, tcp_join_handle)) => {
                        self.push_task(port, udp_tcp_cancel_tx, udp_join_handle, tcp_join_handle);

                        reply.code = ResponseCode::Ok.into();
                        reply.message = String::new();
                        reply.media_server_ip = self.config.my_ip.clone();
                        reply.media_server_port = port as u32;
                        Ok(Response::new(reply))
                    }
                }
            }
        }
    }
}
