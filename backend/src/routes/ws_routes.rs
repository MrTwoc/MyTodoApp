use salvo::prelude::*;
use salvo::websocket::{Message, WebSocket, WebSocketUpgrade};
use tokio::select;
use tokio::time::{Duration, sleep};

use crate::middleware;
use crate::ws;

#[handler]
async fn ws_connect(_depot: &mut Depot, req: &mut Request, res: &mut Response) {
    let mut rx = ws::subscribe();

    if WebSocketUpgrade::new()
        .upgrade(req, res, move |mut socket: WebSocket| async move {
        loop {
            select! {
                msg = socket.recv() => {
                    match msg {
                        Some(Ok(msg)) => {
                            if msg.is_close() {
                                break;
                            }
                        }
                        Some(Err(_)) | None => break,
                    }
                }
                evt = async { rx.recv().await.ok() } => {
                    if let Some(msg) = evt {
                        if socket.send(Message::text(msg)).await.is_err() {
                            break;
                        }
                    }
                }
                _ = sleep(Duration::from_secs(30)) => {
                    if socket.send(Message::ping("ping")).await.is_err() {
                        break;
                    }
                }
            }
        }
        let _ = socket.send(Message::close()).await;
    })
    .await
    .is_err()
    {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(serde_json::json!({
            "error": "WebSocket handshake failed"
        })));
    }
}

pub fn ws_router() -> Router {
    let auth_middleware = middleware::auth::auth_check;

    Router::with_path("ws")
        .hoop(auth_middleware)
        .get(ws_connect)
}
