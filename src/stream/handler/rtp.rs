use std::net::SocketAddr;

use super::StreamHandler;

use crate::stream::utils::reorder::RtpPacketReOrder;

use rtp;

use webrtc_util::Unmarshal;

impl StreamHandler {
    pub fn on_rtp(
        &self,
        _addr: SocketAddr,
        buff: &[u8],
        packets_reorder: &mut RtpPacketReOrder,
    ) -> bool {
        let mut b = buff;
        match rtp::packet::Packet::unmarshal(&mut b) {
            Err(e) => {
                tracing::error!("rtp::packet::Packet::unmarshal error, e: {:?}", e);
                false
            }
            Ok(rtp_packet) => {
                if packets_reorder.feed_rtp(rtp_packet) {
                    let (ts, frame) = packets_reorder.pop_frame();
                    tracing::info!("ts: {}, frame size: {}", ts, frame.len());
                }
                true
            }
        }
    }
}
