use std::process::Command;

use inotify::{EventMask, Inotify, WatchMask};

fn is_webcam_in_use() -> bool {
    let output = Command::new("lsof")
        .arg("/dev/video0")
        .output()
        .expect("Failed to execute command");

    !output.stdout.is_empty()
}

pub async fn start() {
    let webcam_path = "/dev/video0";

    // Create a new inotify instance
    let mut inotify = Inotify::init().expect("Failed to initialize inotify");
    inotify
        .add_watch(webcam_path, WatchMask::OPEN | WatchMask::CLOSE)
        .expect("Failed to add inotify watch");

    tokio::task::spawn(async move {
        let mut buffer = [0; 1024];
        loop {
            let events = inotify
                .read_events_blocking(&mut buffer)
                .expect("Error while reading events");

            for event in events {
                if is_webcam_in_use() {
                    println!("In use!")
                } else {
                    println!("Not used...")
                }
                if event.mask.contains(EventMask::CLOSE_NOWRITE) {
                    println!("Webcam CLOSE_NOWRITE {:?}", std::time::Instant::now());
                }
                if event.mask.contains(EventMask::CLOSE_WRITE) {
                    println!("Webcam CLOSE_WRITE {:?}", std::time::Instant::now());
                }
                if event.mask.contains(EventMask::CREATE) {
                    println!("Webcam CREATE {:?}", std::time::Instant::now());
                }
                if event.mask.contains(EventMask::DELETE) {
                    println!("Webcam DELETE {:?}", std::time::Instant::now());
                }
                if event.mask.contains(EventMask::DELETE_SELF) {
                    println!("Webcam DELETE_SELF {:?}", std::time::Instant::now());
                }
                if event.mask.contains(EventMask::MODIFY) {
                    println!("Webcam MODIFY {:?}", std::time::Instant::now());
                }
                if event.mask.contains(EventMask::MOVE_SELF) {
                    println!("Webcam MOVE_SELF {:?}", std::time::Instant::now());
                }
                if event.mask.contains(EventMask::MOVED_FROM) {
                    println!("Webcam MOVED_FROM {:?}", std::time::Instant::now());
                }
                if event.mask.contains(EventMask::MOVED_TO) {
                    println!("Webcam MOVED_TO {:?}", std::time::Instant::now());
                }
                if event.mask.contains(EventMask::OPEN) {
                    println!("Webcam OPEN {:?}", std::time::Instant::now());
                }
                if event.mask.contains(EventMask::ACCESS) {
                    println!("Webcam ACCESS {:?}", std::time::Instant::now());
                }
                if event.mask.contains(EventMask::IGNORED) {
                    println!("Webcam ACCESS {:?}", std::time::Instant::now());
                }
            }
        }
    })
    .await
    .ok();
}
