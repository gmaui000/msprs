pub mod rtp;

pub struct StreamHandler {
    pub ip: String,
    pub port: u16,
    pub stream_udp_socket: tokio::net::UdpSocket,
    pub stream_tcp_listener: tokio::net::TcpListener,
}

impl StreamHandler {
    pub fn new(
        ip: String,
        port: u16,
        stream_udp_socket: tokio::net::UdpSocket,
        stream_tcp_listener: tokio::net::TcpListener,
    ) -> Self {
        StreamHandler {
            ip,
            port,
            stream_udp_socket,
            stream_tcp_listener,
        }
    }
}
