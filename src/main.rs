pub mod rpc;
pub mod stream;
pub mod utils;
pub mod version;

use clap::Parser;
use std::{path::PathBuf, process::exit};
use tracing::{self, error};
use utils::config::Config;

pub mod gss {
    tonic::include_proto!("gss");
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, value_name = "CONFIG_FILE_PATH")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if !args.config.is_file() {
        error!("config file is not existed: {}", args.config.display());
        exit(1);
    }

    let config = Config::load_from_file(&args.config).unwrap();

    // open daily log
    let _log = utils::log::init(&config);
    // serve grpc
    let rpc_addr = format!("{}:{}", &config.host, &config.grpc_port);
    let rpc_service = rpc::server::MyGbtStreamService::new(config.clone());
    match tonic::transport::Server::builder()
        .add_service(gss::gbt_stream_service_server::GbtStreamServiceServer::new(
            rpc_service,
        ))
        .serve(rpc_addr.parse().unwrap())
        .await
    {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("grpc serve error, e: {:?}", e);
        }
    };

    Ok(())
}
