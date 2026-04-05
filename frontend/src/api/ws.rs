use crate::api::ApiClient;
use crate::store::get_local_storage_item;
use leptos::prelude::Callable;
use leptos::prelude::Callback;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;
use web_sys::{CloseEvent, ErrorEvent, Event, MessageEvent, WebSocket};

const TOKEN_KEY: &str = "todo_token";

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
    _open: Closure<dyn Fn(Event)>,
    _message: Closure<dyn Fn(MessageEvent)>,
    _error: Closure<dyn Fn(ErrorEvent)>,
    _close: Closure<dyn Fn(CloseEvent)>,
    is_valid: Rc<RefCell<bool>>,
}

impl WsConnection {
    pub fn send(&self, text: &str) -> Result<(), String> {
        self.socket
            .send_with_str(text)
            .map_err(|e| format!("{:?}", e))
    }

    pub fn close(&self) {
        *self.is_valid.borrow_mut() = false;
        let _ = self.socket.close();
    }

    pub fn is_open(&self) -> bool {
        self.socket.ready_state() == WebSocket::OPEN && *self.is_valid.borrow()
    }

    pub fn is_valid(&self) -> bool {
        *self.is_valid.borrow()
    }
}

impl Drop for WsConnection {
    fn drop(&mut self) {
        *self.is_valid.borrow_mut() = false;
        let _ = self.socket.close();
    }
}

unsafe impl Send for WsConnection {}

unsafe impl Sync for WsConnection {}

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

fn get_token() -> Option<String> {
    get_local_storage_item(TOKEN_KEY)
}

pub fn build_ws_url(base_url: &str, token: Option<&str>) -> String {
    let ws_base = normalize_base_url(base_url);
    let base = format!("{}/ws", ws_base.trim_end_matches('/'));
    match token {
        Some(t) => format!("{}?token={}", base, t),
        None => base,
    }
}

pub fn connect_ws(
    client: &ApiClient,
    on_state: Callback<(WsState,)>,
    on_message: Callback<(WsEvent,)>,
) -> Result<WsConnection, String> {
    let token = get_token();
    let url = build_ws_url(client.base_url(), token.as_deref());
    web_sys::console::log_1(&format!("[WS] Connecting to: {}", url).into());
    let socket =
        WebSocket::new(&url).map_err(|e| format!("Failed to create WebSocket: {:?}", e))?;

    let is_valid = Rc::new(RefCell::new(true));
    let is_valid_open = is_valid.clone();
    let is_valid_close = is_valid.clone();
    let is_valid_error = is_valid.clone();
    let is_valid_message = is_valid.clone();

    let open_callback = {
        let on_state = on_state.clone();
        Closure::wrap(Box::new(move |_event: Event| {
            if !*is_valid_open.borrow() {
                return;
            }
            web_sys::console::log_1(&"[WS] Connection opened".into());
            on_state.run((WsState::Open,));
        }) as Box<dyn Fn(Event)>)
    };

    let close_callback = {
        let on_state = on_state.clone();
        Closure::wrap(Box::new(move |_event: CloseEvent| {
            if !*is_valid_close.borrow() {
                return;
            }
            *is_valid_close.borrow_mut() = false;
            web_sys::console::log_1(&"[WS] Connection closed".into());
            on_state.run((WsState::Closed,));
        }) as Box<dyn Fn(CloseEvent)>)
    };

    let error_callback = {
        let on_state = on_state.clone();
        Closure::wrap(Box::new(move |_event: ErrorEvent| {
            if !*is_valid_error.borrow() {
                return;
            }
            web_sys::console::log_1(&"[WS] Connection error".into());
            on_state.run((WsState::Error("WebSocket error".to_string()),));
        }) as Box<dyn Fn(ErrorEvent)>)
    };

    let message_callback = {
        let on_message = on_message;
        Closure::wrap(Box::new(move |event: MessageEvent| {
            if !*is_valid_message.borrow() {
                return;
            }
            web_sys::console::log_1(&"[WS] Message received".into());
            let data = event.data();
            if let Ok(text) = data.dyn_into::<js_sys::JsString>() {
                let msg_str = String::from(text);
                web_sys::console::log_1(
                    &format!("[WS] Raw message: {}", &msg_str[..msg_str.len().min(100)]).into(),
                );
                let payload = parse_text_or_raw(&msg_str);
                on_message.run((payload,));
            }
        }) as Box<dyn Fn(MessageEvent)>)
    };

    socket.set_onopen(Some(open_callback.as_ref().unchecked_ref()));
    socket.set_onmessage(Some(message_callback.as_ref().unchecked_ref()));
    socket.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
    socket.set_onclose(Some(close_callback.as_ref().unchecked_ref()));

    web_sys::console::log_1(&"[WS] Setting state to Connecting".into());
    on_state.run((WsState::Connecting,));

    web_sys::console::log_1(&"[WS] Returning connection".into());

    Ok(WsConnection {
        socket,
        _open: open_callback,
        _message: message_callback,
        _error: error_callback,
        _close: close_callback,
        is_valid,
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
