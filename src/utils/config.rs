use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_my_ip")]
    pub my_ip: String,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,
    #[serde(default = "default_stream_port_start")]
    pub stream_port_start: u16,
    #[serde(default = "default_stream_port_stop")]
    pub stream_port_stop: u16,
    #[serde(default = "default_socket_recv_buffer_size")]
    pub socket_recv_buffer_size: usize,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_my_ip() -> String {
    "".to_string()
}

fn default_grpc_port() -> u16 {
    7080
}

fn default_stream_port_start() -> u16 {
    10001
}

fn default_stream_port_stop() -> u16 {
    20000
}

fn default_socket_recv_buffer_size() -> usize {
    1500
}

impl Config {
    // 从YAML文件加载配置
    pub fn load_from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let yaml_content = fs::read_to_string(path)?;
        let mut config: Config = serde_yaml::from_str(&yaml_content)?;
        if config.my_ip.is_empty() {
            if let Ok(ip) = local_ip() {
                config.my_ip = ip.to_string();
            }
        }

        Ok(config)
    }

    // 保存配置到YAML文件（示例，可按需完善）
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let yaml_content = serde_yaml::to_string(self)?;
        fs::write(path, yaml_content)?;
        Ok(())
    }
}
