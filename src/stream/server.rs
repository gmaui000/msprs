use tokio::{self, io::AsyncReadExt};

use super::handler::StreamHandler;
use super::utils::reorder::RtpPacketReOrder;

pub async fn bind(
    host: &String,
    port: u16,
) -> Result<(tokio::net::UdpSocket, tokio::net::TcpListener), std::io::Error> {
    let local_addr = format!("{host}:{port}",);

    // udp server
    match tokio::net::UdpSocket::bind(&local_addr).await {
        Err(e) => {
            tracing::error!("UdpSocket::bind({}) error, e: {:?}", &local_addr, e);
            Err(e)
        }
        Ok(udp_socket) => {
            tracing::info!("UdpSocket::bind({}) ok", &local_addr);

            // tcp server
            match tokio::net::TcpListener::bind(&local_addr).await {
                Err(e) => {
                    tracing::error!("TcpListener::bind({}) error, e: {:?}", &local_addr, e);
                    Err(e)
                }
                Ok(tcp_listener) => {
                    tracing::info!("TcpListener::bind({}) ok", &local_addr);
                    Ok((udp_socket, tcp_listener))
                }
            }
        }
    }
}

pub async fn run_forever(
    cancel_tx: tokio::sync::broadcast::Sender<()>,
    socket_recv_buffer_size: usize,
    stream_handler: std::sync::Arc<StreamHandler>,
) -> Result<(tokio::task::JoinHandle<()>, tokio::task::JoinHandle<()>), std::io::Error> {
    // udp server
    let mut udp_cancel_rx = cancel_tx.subscribe();
    let udp_stream_handler = stream_handler.clone();
    let udp_join_handle = tokio::spawn(async move {
        tracing::info!(
            "udp stream service start, port: {}",
            udp_stream_handler.port
        );

        let mut recv_buff = Vec::<u8>::default();
        recv_buff.resize(socket_recv_buffer_size, 0);

        let mut packets_reorder = RtpPacketReOrder::new(3, "udp.output.ps");

        loop {
            tokio::select! {
                _ = udp_cancel_rx.recv() => {
                    tracing::warn!("cancel udp recv_from");
                    break;
                }
                result = udp_stream_handler.stream_udp_socket.recv_from(recv_buff.as_mut_slice()) => {
                    match result {
                        Err(e) => {
                            tracing::error!("UdpSocket::recv_from error, e: {:?}", e);
                            break;
                        }
                        Ok((amount, addr)) => {
                            // dispatch rtp data
                            udp_stream_handler.on_rtp(
                                addr,
                                &recv_buff.as_slice()[..amount],
                                &mut packets_reorder,
                            );
                        }
                    }
                }
            }
        }

        tracing::info!("udp stream service stop, port: {}", udp_stream_handler.port);
    });

    // tcp server
    let mut tcp_cancel_accept_rx = cancel_tx.subscribe();
    let mut tcp_cancel_read_u16_rx = cancel_tx.subscribe();
    let mut tcp_cancel_read_extract_rx = cancel_tx.subscribe();
    let tcp_stream_handler = stream_handler.clone();
    let tcp_join_handle = tokio::spawn(async move {
        tracing::info!(
            "tcp stream service start, port: {}",
            tcp_stream_handler.port
        );

        loop {
            tokio::select! {
                _ = tcp_cancel_accept_rx.recv() => {
                    tracing::warn!("cancel tcp accept");
                    break;
                }
                accept_result = tcp_stream_handler.stream_tcp_listener.accept() => {
                    match accept_result {
                        Err(e) => {
                            tracing::error!("TcpListener::accept error, e: {:?}", e);
                            continue;
                        }
                        Ok((mut tcp_stream, addr)) => {
                            let mut packets_reorder = RtpPacketReOrder::new(3, "tcp.output.ps");

                            loop {
                                tokio::select! {
                                    _ = tcp_cancel_read_u16_rx.recv() => {
                                        tracing::warn!("cancel tcp read_u16");
                                        break;
                                    }
                                    u16_result = tcp_stream.read_u16() => {
                                        // read 2 bytes size header
                                        match u16_result {
                                            Err(e) => {
                                                tracing::error!("TcpStream::read_u16 error, e: {:?}", e);
                                                break;
                                            }
                                            Ok(n) => {
                                                // read n bytes content
                                                let mut recv_buff = vec![0; n as usize];
                                                tokio::select! {
                                                    _ = tcp_cancel_read_extract_rx.recv() => {
                                                        tracing::warn!("cancel tcp read_extract");
                                                        break;
                                                    }
                                                    exact_result = tcp_stream.read_exact(&mut recv_buff) => {
                                                        match exact_result {
                                                            Ok(0) => {
                                                                tracing::error!("tcp connection closed");
                                                                break;
                                                            }
                                                            Err(e) => {
                                                                tracing::error!("TcpStream::read error, e: {:?}", e);
                                                                break;
                                                            }
                                                            Ok(amount) => {
                                                                // dispatch rtp data
                                                                tcp_stream_handler.on_rtp(
                                                                    addr,
                                                                    &recv_buff.as_slice()[..amount],
                                                                    &mut packets_reorder,
                                                                );
                                                            }
                                                        }
                                                    }
                                                };
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        tracing::info!("tcp stream service stop, port: {}", tcp_stream_handler.port);
    });

    Ok((udp_join_handle, tcp_join_handle))
}
