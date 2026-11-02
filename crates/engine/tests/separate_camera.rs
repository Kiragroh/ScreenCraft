#![cfg(windows)]
use screencraft_audio::Clock;
use screencraft_capture::camera::{CameraHub, CamFrame};
use screencraft_engine::{camera_rec::CameraRecorder, FrameTable, Package};
use screencraft_encode::HwPreference;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[test]
fn separate_camera_is_playable_synced_and_preserves_screen_and_input() {
    let root = std::env::temp_dir().join(format!("screencraft-camera-test-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    let pkg = Package::create_in(&root, "synthetic").unwrap();
    std::fs::write(pkg.dir.join("screen.mp4"), b"unchanged screen").unwrap();
    std::fs::write(pkg.dir.join("input.json"), br#"{"moves":[],"clicks":[]}"#).unwrap();
    let hub = CameraHub::new();
    let clock = Clock::start();
    let camera_start = clock.epoch() + Duration::from_millis(50);
    let producer_hub = hub.clone();
    let producer = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        for i in 0..12 {
            producer_hub.publish(CamFrame { width: 160, height: 120,
                rgb: vec![(i * 15) as u8; 160 * 120 * 3],
                at: camera_start + Duration::from_millis(i * 34) });
            std::thread::sleep(Duration::from_millis(50));
        }
    });
    let rec = CameraRecorder::start(hub, pkg.dir.join("camera.mp4"), clock, HwPreference::Software).unwrap();
    producer.join().unwrap();
    assert_eq!(rec.finish(0).unwrap(), -50.0);
    let table = FrameTable::open(&pkg.dir.join("camera.mp4")).unwrap();
    assert_eq!(table.len(), 12); assert_eq!((table.width, table.height), (160, 120));
    assert_eq!(std::fs::read(pkg.dir.join("screen.mp4")).unwrap(), b"unchanged screen");
    assert_eq!(std::fs::read(pkg.dir.join("input.json")).unwrap(), br#"{"moves":[],"clicks":[]}"#);
    std::fs::remove_dir_all(root).unwrap();
}
