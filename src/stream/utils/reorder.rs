use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;

use rtp;

pub struct RtpPacketReOrder {
    min_timestamp: u32,
    limit_frames: usize,
    packet_groups: BTreeMap<u32, BTreeMap<u16, rtp::packet::Packet>>, // timestamp -> {sequence_number -> packet}
    output_file: Option<std::fs::File>,
}

impl RtpPacketReOrder {
    pub fn new(limit_frames: usize, output_file: &str) -> Self {
        let file = if output_file.is_empty() {
            None
        } else {
            Some(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(output_file)
                    .unwrap(),
            )
        };

        RtpPacketReOrder {
            min_timestamp: 0,
            limit_frames,
            packet_groups: BTreeMap::new(),
            output_file: file,
        }
    }

    pub fn feed_rtp(&mut self, packet: rtp::packet::Packet) -> bool {
        let timestamp = packet.header.timestamp;
        let sequence_number = packet.header.sequence_number;

        // drop
        if timestamp < self.min_timestamp {
            tracing::warn!("expired packet, {} < {}", timestamp, self.min_timestamp);
            return false;
        }

        // first level tree (use timestamp as key)
        let timestamp_tree = self.packet_groups.entry(timestamp).or_default();

        // second level tree (use sequence_number as key)
        timestamp_tree.entry(sequence_number).or_insert(packet);

        self.packet_groups.len() > self.limit_frames
    }

    pub fn pop_frame(&mut self) -> (u32, Vec<u8>) {
        // pop minimum tree, merge its packet payload into a frame
        let mut ts = 0;
        let mut frame = Vec::new();
        if let Some((key, timestamp_tree)) = self.packet_groups.pop_first() {
            ts = key;
            for (_sn, packet) in timestamp_tree {
                frame.extend_from_slice(&packet.payload);
            }

            // write frame to file
            if let Some(ref mut file) = self.output_file {
                if let Err(e) = file.write_all(&frame) {
                    tracing::error!("std::io::write_all error, e: {:?}", e);
                }
            }
        }

        // update min_timestamp
        if !self.packet_groups.is_empty() {
            self.min_timestamp = *self.packet_groups.keys().min().unwrap_or(&0);
        }

        (ts, frame)
    }
}
