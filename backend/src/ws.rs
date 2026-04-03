use std::sync::OnceLock;

use serde_json::{json, Value};
use tokio::sync::broadcast::{self, Receiver, Sender};

static BROADCASTER: OnceLock<Sender<String>> = OnceLock::new();

const CHANNEL_CAPACITY: usize = 512;

fn broadcaster() -> &'static Sender<String> {
    BROADCASTER.get_or_init(|| {
        let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
        tx
    })
}

pub fn subscribe() -> Receiver<String> {
    broadcaster().subscribe()
}

pub fn push(event: &str, payload: Value) {
    let message = json!({
        "event": event,
        "timestamp": chrono::Utc::now().timestamp(),
        "payload": payload,
    });

    let _ = broadcaster().send(message.to_string());
}
