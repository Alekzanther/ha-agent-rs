use std::{process::Command};
use tokio::sync::watch::Sender;

use inotify::{Inotify, WatchMask};

pub fn is_webcam_in_use() -> bool {
    let output = Command::new("lsof")
        .arg("/dev/video0")
        .output()
        .expect("Failed to execute command");

    !output.stdout.is_empty()
}

pub async fn start(webcam_state_tx: Sender<bool>) {
    let webcam_path = "/dev/video0";

    // Create a new inotify instance
    let mut inotify = Inotify::init().expect("Failed to initialize inotify");
    inotify
        .add_watch(webcam_path, WatchMask::OPEN | WatchMask::CLOSE)
        .expect("Failed to add inotify watch");
    let mut webcam_in_use = false;

    let mut buffer = [0; 1024];
    loop {
        let events = inotify
            .read_events_blocking(&mut buffer)
            .expect("Error while reading events");

        for _event in events {
            if is_webcam_in_use() != webcam_in_use {
                webcam_in_use = !webcam_in_use;
                webcam_state_tx.send(webcam_in_use).expect("Unable to send");
            }
        }
    }
}
