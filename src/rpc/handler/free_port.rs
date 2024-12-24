use tonic::{Request, Response, Status};

use crate::gss::{FreeStreamPortRequest, FreeStreamPortResponse, ResponseCode};
use crate::rpc::server::MyGbtStreamService;

impl MyGbtStreamService {
    pub async fn rpc_free_stream_port(
        &self,
        request: Request<FreeStreamPortRequest>,
    ) -> Result<Response<FreeStreamPortResponse>, Status> {
        let req = request.into_inner();
        let port = req.media_server_port as u16;

        self.pop_task(port).await;
        self.push_port(port);

        let reply = FreeStreamPortResponse {
            code: ResponseCode::Ok.into(),
            ..Default::default()
        };

        Ok(Response::new(reply))
    }
}
