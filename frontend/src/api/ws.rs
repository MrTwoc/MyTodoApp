use crate::api::ApiClient;
use leptos::prelude::Callable;
use leptos::prelude::Callback;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;
use web_sys::{CloseEvent, ErrorEvent, Event, MessageEvent, WebSocket};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsEvent {
    pub event: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum WsState {
    Connecting,
    Open,
    Closed,
    Error(String),
}

pub struct WsConnection {
    socket: WebSocket,
    _open: Closure<dyn FnMut(Event)>,
    _message: Closure<dyn FnMut(MessageEvent)>,
    _error: Closure<dyn FnMut(ErrorEvent)>,
    _close: Closure<dyn FnMut(CloseEvent)>,
}

impl WsConnection {
    pub fn send(&self, text: &str) -> Result<(), String> {
        self.socket
            .send_with_str(text)
            .map_err(|e| format!("{:?}", e))
    }

    pub fn close(&self) {
        let _ = self.socket.close();
    }

    pub fn is_open(&self) -> bool {
        self.socket.ready_state() == WebSocket::OPEN
    }
}

impl Drop for WsConnection {
    fn drop(&mut self) {
        let _ = self.socket.close();
    }
}

fn normalize_base_url(base_url: &str) -> String {
    let trimmed = base_url.trim_end_matches('/');
    if trimmed.starts_with("ws://") || trimmed.starts_with("wss://") {
        return trimmed.to_string();
    }
    if trimmed.starts_with("https://") {
        return trimmed.replacen("https://", "wss://", 1);
    }
    if trimmed.starts_with("http://") {
        return trimmed.replacen("http://", "ws://", 1);
    }
    trimmed.to_string()
}

pub fn build_ws_url(base_url: &str) -> String {
    let ws_base = normalize_base_url(base_url);
    format!("{}/ws", ws_base.trim_end_matches('/'))
}

pub fn connect_ws(
    client: &ApiClient,
    on_state: Callback<WsState>,
    on_message: Callback<WsEvent>,
) -> Result<WsConnection, String> {
    let url = build_ws_url(client.base_url());
    let socket =
        WebSocket::new(&url).map_err(|e| format!("Failed to create WebSocket: {:?}", e))?;

    let open_callback = {
        let on_state = on_state;
        Closure::wrap(Box::new(move |_event: Event| {
            on_state.run(WsState::Open);
        }) as Box<dyn FnMut(Event)>)
    };

    let close_callback = {
        let on_state = on_state;
        Closure::wrap(Box::new(move |_event: CloseEvent| {
            on_state.run(WsState::Closed);
        }) as Box<dyn FnMut(CloseEvent)>)
    };

    let error_callback = {
        let on_state = on_state;
        Closure::wrap(Box::new(move |_event: ErrorEvent| {
            on_state.run(WsState::Error("WebSocket error".to_string()));
        }) as Box<dyn FnMut(ErrorEvent)>)
    };

    let message_callback = {
        let on_message = on_message;
        Closure::wrap(Box::new(move |event: MessageEvent| {
            let data = event.data();
            if let Ok(text) = data.dyn_into::<js_sys::JsString>() {
                let payload = parse_text_or_raw(&String::from(text));
                on_message.run(payload);
            }
        }) as Box<dyn FnMut(MessageEvent)>)
    };

    socket.set_onopen(Some(open_callback.as_ref().unchecked_ref()));
    socket.set_onmessage(Some(message_callback.as_ref().unchecked_ref()));
    socket.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
    socket.set_onclose(Some(close_callback.as_ref().unchecked_ref()));

    on_state.run(WsState::Connecting);

    Ok(WsConnection {
        socket,
        _open: open_callback,
        _message: message_callback,
        _error: error_callback,
        _close: close_callback,
    })
}

fn parse_text_or_raw(text: &str) -> WsEvent {
    let payload = if let Ok(event) = serde_json::from_str::<serde_json::Value>(text) {
        if let Some(event_name) = event.get("event").and_then(|v| v.as_str()) {
            WsEvent {
                event: event_name.to_string(),
                payload: event
                    .get("payload")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
            }
        } else {
            WsEvent {
                event: "raw".to_string(),
                payload: serde_json::Value::String(text.to_string()),
            }
        }
    } else {
        WsEvent {
            event: "raw".to_string(),
            payload: serde_json::Value::String(text.to_string()),
        }
    };

    payload
}
