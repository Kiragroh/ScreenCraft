#![cfg(target_os = "windows")]

use screencraft_encode::{create_video_encoder, I420, VideoCodec, VideoConfig};

/// The MP4 writer has one timestamp per sample and cannot represent reordered B frames.
#[test]
fn windows_h264_preserves_presentation_order_and_all_frames() {
    let cfg = VideoConfig::new(VideoCodec::H264, 320, 240, 30);
    let mut encoder = create_video_encoder(&cfg).expect("Windows H.264 encoder");
    let mut packets = Vec::new();
    for index in 0..60 {
        let frame = I420 {
            width: 320,
            height: 240,
            y: vec![32 + index as u8 * 2; 320 * 240],
            u: vec![128; 160 * 120],
            v: vec![128; 160 * 120],
        };
        packets.extend(encoder.encode(&frame, index * 1_000_000 / 30).expect("encode frame"));
    }
    packets.extend(encoder.flush().expect("flush all frames"));
    let actual: Vec<i64> = packets.iter().map(|packet| packet.pts_us).collect();
    let expected: Vec<i64> = (0..60).map(|index| index * 1_000_000 / 30).collect();
    assert_eq!(actual, expected, "output must preserve every input timestamp in order");
}
