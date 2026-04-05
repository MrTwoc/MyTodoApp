use crate::api::dashboard::get_overview;
use crate::api::ws::{WsConnection, WsEvent, WsState, connect_ws};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::card::{Card, CardFooter};
use crate::components::loading::{Loading, LoadingVariant};
use crate::components::task_card::TaskCard;
use crate::store::{use_api_client, use_user_store};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use serde_json::Value;
use std::sync::{Arc, Mutex};
fn stat_label(total: u32) -> &'static str {
    if total == 0 {
        "No data"
    } else if total == 1 {
        "One item"
    } else {
        "Items"
    }
}

fn task_payload_to_text(payload: &Value) -> String {
    match serde_json::to_string(payload) {
        Ok(raw) => raw,
        Err(_) => String::from("unreadable payload"),
    }
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    let client = use_api_client();
    let navigate = use_navigate();
    let user_store = use_user_store();

    let (overview, set_overview) = signal(None::<crate::api::dashboard::DashboardOverview>);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (ws_status, set_ws_status) = signal(String::from("Disconnected"));
    let (ws_count, set_ws_count) = signal(0_u32);
    let (ws_logs, set_ws_logs) = signal(Vec::<String>::new());
    let (ws_conn, set_ws_conn) = signal(Option::<Arc<Mutex<WsConnection>>>::None);
    let (initialized, set_initialized) = signal(false);

    let personal_stats = move || {
        overview
            .get()
            .map(|data| data.personal_tasks)
            .unwrap_or_default()
    };
    let team_stats = move || {
        overview
            .get()
            .map(|data| data.team_tasks)
            .unwrap_or_default()
    };

    let user_label = move || {
        overview
            .get()
            .and_then(|d| d.username)
            .or_else(|| user_store.profile().map(|u| u.username))
            .unwrap_or_else(|| "there".to_string())
    };

    let load_dashboard: Callback<()> = {
        let client = client.clone();
        let set_error = set_error;
        let set_loading = set_loading;
        let set_overview = set_overview;
        Callback::from(move || {
            set_error.set(None);
            set_loading.set(true);
            let client_clone = client.clone();
            let set_error_clone = set_error.clone();
            let set_loading_clone = set_loading.clone();
            let set_overview_clone = set_overview.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match get_overview(&client_clone).await {
                    Ok(result) => {
                        set_overview_clone.set(Some(result));
                    }
                    Err(err) => {
                        set_error_clone.set(Some(err.message));
                    }
                }
                set_loading_clone.set(false);
            });
        })
    };

    let connect_realtime: Callback<()> = {
        let client = client.clone();
        let set_ws_status = set_ws_status;
        let set_ws_count = set_ws_count;
        let set_ws_logs = set_ws_logs;
        let ws_conn = ws_conn;
        let set_ws_conn = set_ws_conn;
        Callback::from(move || {
            if ws_conn.get_untracked().is_some() {
                return;
            }

            let set_ws_status_inner = set_ws_status.clone();
            let set_ws_logs_inner = set_ws_logs.clone();
            let set_ws_count_inner = set_ws_count.clone();

            match connect_ws(&client, 
                Callback::new(move |(state,): (WsState,)| {
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                        match state {
                            WsState::Connecting => set_ws_status_inner.set("Connecting".to_string()),
                            WsState::Open => set_ws_status_inner.set("Connected".to_string()),
                            WsState::Closed => set_ws_status_inner.set("Closed".to_string()),
                            WsState::Error(msg) => set_ws_status_inner.set(format!("Error: {msg}")),
                        }
                    }));
                }),
                Callback::new(move |(evt,): (WsEvent,)| {
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                        let line = if evt.event.is_empty() {
                            format!("raw: {}", task_payload_to_text(&evt.payload))
                        } else {
                            format!("{}: {}", evt.event, task_payload_to_text(&evt.payload))
                        };

                        set_ws_logs_inner.update(|items| {
                            items.insert(0, line);
                            if items.len() > 5 {
                                items.truncate(5);
                            }
                        });
                        set_ws_count_inner.update(|value| *value = value.saturating_add(1));
                    }));
                })
            ) {
                Ok(conn) => {
                    set_ws_conn.set(Some(Arc::new(Mutex::new(conn))));
                }
                Err(msg) => {
                    set_ws_status.set(format!("Error: {msg}"));
                }
            }
        })
    };

    let refresh_button = Callback::from(move |_| {
        load_dashboard.run(());
    });

    let nav_tasks = {
        let n = navigate.clone();
        Callback::from(move |_| n("/tasks", Default::default()))
    };
    let nav_teams = {
        let n = navigate.clone();
        Callback::from(move |_| n("/teams", Default::default()))
    };
    let nav_settings = {
        let n = navigate.clone();
        Callback::from(move |_| n("/settings", Default::default()))
    };

    Effect::new(move |_| {
        if !initialized.get() {
            set_initialized.set(true);
            load_dashboard.run(());
            connect_realtime.run(());
        }
    });

    on_cleanup(move || {
        if let Some(conn) = ws_conn.get_untracked() {
            conn.lock().unwrap().close();
        }
        set_ws_conn.set(None);
    });

    view! {
        <div class="dashboard">
            <header class="dashboard-header">
                <div class="dashboard-header-left">
                    <h1 class="dashboard-title">"Dashboard"</h1>
                    <p class="dashboard-greeting">
                        {move || format!("Welcome, {}", user_label())}
                    </p>
                </div>
                <div class="dashboard-actions">
                    <Button variant=ButtonVariant::Secondary size=ButtonSize::Sm on_click=nav_tasks>"Tasks"</Button>
                    <Button variant=ButtonVariant::Secondary size=ButtonSize::Sm on_click=nav_teams>"Teams"</Button>
                    <Button variant=ButtonVariant::Secondary size=ButtonSize::Sm on_click=nav_settings>"Settings"</Button>
                </div>
            </header>

            <div class="dashboard-main">
                <Card title="Real-time Feed".to_string() subtitle="WebSocket notifications".to_string()>
                    <div class="dashboard-ws">
                        <div class="dashboard-ws-header">
                            <div class="dashboard-ws-status-row">
                                <span class=move || {
                                    let status = ws_status.get();
                                    let status_lower = status.to_lowercase();
                                    if status_lower.contains("connected") {
                                        "dashboard-ws-status connected"
                                    } else if status_lower.contains("connecting") {
                                        "dashboard-ws-status connecting"
                                    } else if status_lower.contains("error") {
                                        "dashboard-ws-status error"
                                    } else {
                                        "dashboard-ws-status"
                                    }
                                }>
                                    {move || format!("Status: {}", ws_status.get())}
                                </span>
                                <span class="dashboard-ws-count">
                                    {move || format!("{} events", ws_count.get())}
                                </span>
                            </div>
                        </div>
                        <div class="dashboard-ws-log">
                            {move || {
                                let logs = ws_logs.get();
                                if logs.is_empty() {
                                    view! { <p class="empty-text">"No events yet. Waiting for WebSocket messages..."</p> }.into_any()
                                } else {
                                    let items = logs
                                        .into_iter()
                                        .map(|msg| view! { <p class="dashboard-ws-item">{msg}</p> })
                                        .collect::<Vec<_>>();
                                    view! { <div>{items}</div> }.into_any()
                                }
                            }}
                        </div>
                    </div>
                    <CardFooter>
                        <Button
                            variant=ButtonVariant::Secondary
                            size=ButtonSize::Sm
                            on_click=refresh_button
                        >
                            "Refresh"
                        </Button>
                    </CardFooter>
                </Card>

                <div class="dashboard-grid">
                    <Card title="Personal Task Board".to_string() subtitle="Your current status".to_string()>
                        <div class="stat-row">
                            <div class="stat stat-total">
                                <span class="stat-number">{move || personal_stats().total}</span>
                                <span class="stat-label">{move || stat_label(personal_stats().total)}</span>
                            </div>
                            <div class="stat stat-active">
                                <span class="stat-number">{move || personal_stats().active}</span>
                                <span class="stat-label">"Active"</span>
                            </div>
                            <div class="stat stat-completed">
                                <span class="stat-number">{move || personal_stats().completed}</span>
                                <span class="stat-label">"Completed"</span>
                            </div>
                            <div class="stat stat-paused">
                                <span class="stat-number">{move || personal_stats().paused}</span>
                                <span class="stat-label">"Paused"</span>
                            </div>
                        </div>
                        <CardFooter>
                            <Button variant=ButtonVariant::Primary size=ButtonSize::Sm on_click=nav_tasks>
                                "Open Personal Tasks"
                            </Button>
                        </CardFooter>
                    </Card>

                    <Card title="Team Task Board".to_string() subtitle="Team progress".to_string()>
                        <div class="stat-row">
                            <div class="stat stat-total">
                                <span class="stat-number">{move || team_stats().total}</span>
                                <span class="stat-label">{move || stat_label(team_stats().total)}</span>
                            </div>
                            <div class="stat stat-active">
                                <span class="stat-number">{move || team_stats().active}</span>
                                <span class="stat-label">"Active"</span>
                            </div>
                            <div class="stat stat-completed">
                                <span class="stat-number">{move || team_stats().completed}</span>
                                <span class="stat-label">"Completed"</span>
                            </div>
                            <div class="stat stat-paused">
                                <span class="stat-number">{move || team_stats().paused}</span>
                                <span class="stat-label">"Paused"</span>
                            </div>
                        </div>
                        <CardFooter>
                            <Button variant=ButtonVariant::Primary size=ButtonSize::Sm on_click=nav_teams>
                                "Open Teams"
                            </Button>
                        </CardFooter>
                    </Card>
                </div>
            </div>
        </div>
    }
}
