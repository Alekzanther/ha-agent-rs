use std::process::Command;
use std::str;
use tokio::sync::watch::Sender;
use std::time::Duration;

use tokio::time::sleep;

pub fn is_microphone_in_use() -> bool {
    let microphone_matcher = "input";
    let output = Command::new("pactl")
        .args(["list", "short", "sources"])
        .output()
        .expect("Failed to execute command");

    let output_str = str::from_utf8(&output.stdout).unwrap();

    output_str
        .lines()
        .filter(|line| line.contains("RUNNING") && line.contains(microphone_matcher))
        .count()
        > 0
}

pub async fn start(microphone_state_tx: Sender<bool>) {
    // Create a new inotify instance
    let mut microphone_in_use = false;

    loop {
        if is_microphone_in_use() != microphone_in_use {
            microphone_in_use = !microphone_in_use;
            microphone_state_tx.send(microphone_in_use).expect("Unable to send");
        }
        sleep(Duration::from_secs(5)).await;
    }
}
