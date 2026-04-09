use crate::api::dashboard::get_overview;
use crate::api::ws::{WsConnection, WsEvent, WsState, connect_ws};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::card::{Card, CardFooter};
use crate::components::loading::{Loading, LoadingVariant};
use crate::components::task_card::{TaskCard, TaskCardVariant};
use crate::store::{use_api_client, use_user_store};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use serde_json::Value;
use std::sync::{Arc, Mutex};

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

    let total_tasks = move || {
        let p = personal_stats();
        let t = team_stats();
        p.total + t.total
    };

    let completed_tasks = move || {
        let p = personal_stats();
        let t = team_stats();
        p.completed + t.completed
    };

    let recent_personal_tasks = move || {
        overview
            .get()
            .map(|data| data.recent_personal_tasks.clone())
            .unwrap_or_default()
    };

    let recent_team_tasks = move || {
        overview
            .get()
            .map(|data| data.recent_team_tasks.clone())
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
        Callback::from(move |_: leptos::ev::MouseEvent| { n("/tasks", Default::default()) })
    };
    let nav_teams = {
        let n = navigate.clone();
        Callback::from(move |_: leptos::ev::MouseEvent| { n("/teams", Default::default()) })
    };
    let nav_tasks_clone = nav_tasks.clone();
    let nav_teams_clone = nav_teams.clone();

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
            <div class="dashboard-header">
                <div class="dashboard-header-main">
                    <h1 class="dashboard-page-title">"Welcome back"</h1>
                    <p class="dashboard-page-subtitle">
                        {user_label()} <span class="dashboard-date"></span>
                    </p>
                </div>
                <div class="dashboard-header-actions">
                    <Button variant=ButtonVariant::Primary size=ButtonSize::Md on_click=nav_tasks>
                        "View Tasks"
                    </Button>
                    <Button variant=ButtonVariant::Secondary size=ButtonSize::Md on_click=nav_teams>
                        "Teams"
                    </Button>
                </div>
            </div>

            {move || {
                if loading.get() {
                    view! {
                        <div class="dashboard-loading">
                            <Loading variant=LoadingVariant::Spinner label="Loading...".to_string() size=40 />
                        </div>
                    }.into_any()
                } else if let Some(err) = error.get() {
                    view! {
                        <div class="dashboard-error">
                            <div class="error-icon">!</div>
                            <p>{err}</p>
                            <Button variant=ButtonVariant::Primary size=ButtonSize::Md on_click=refresh_button>
                                "Retry"
                            </Button>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="dashboard-main">
                            <div class="stats-cards">
                                <div class="stat-mini-card">
                                    <div class="stat-mini-icon total">
                                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                            <path d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2"/>
                                            <rect x="9" y="3" width="6" height="4" rx="1"/>
                                            <path d="M9 12h6M9 16h6"/>
                                        </svg>
                                    </div>
                                    <div class="stat-mini-content">
                                        <span class="stat-mini-value">{total_tasks()}</span>
                                        <span class="stat-mini-label">"Total Tasks"</span>
                                    </div>
                                </div>

                                <div class="stat-mini-card">
                                    <div class="stat-mini-icon active">
                                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                            <circle cx="12" cy="12" r="10"/>
                                            <path d="M12 6v6l4 2"/>
                                        </svg>
                                    </div>
                                    <div class="stat-mini-content">
                                        <span class="stat-mini-value">{personal_stats().active + team_stats().active}</span>
                                        <span class="stat-mini-label">"In Progress"</span>
                                    </div>
                                </div>

                                <div class="stat-mini-card">
                                    <div class="stat-mini-icon completed">
                                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                            <path d="M22 11.08V12a10 10 0 11-5.93-9.14"/>
                                            <path d="M22 4L12 14.01l-3-3"/>
                                        </svg>
                                    </div>
                                    <div class="stat-mini-content">
                                        <span class="stat-mini-value">{completed_tasks()}</span>
                                        <span class="stat-mini-label">"Completed"</span>
                                    </div>
                                </div>

                                <div class="stat-mini-card">
                                    <div class="stat-mini-icon personal">
                                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                                            <circle cx="12" cy="7" r="4"/>
                                        </svg>
                                    </div>
                                    <div class="stat-mini-content">
                                        <span class="stat-mini-value">{personal_stats().total}</span>
                                        <span class="stat-mini-label">"Personal"</span>
                                    </div>
                                </div>

                                <div class="stat-mini-card">
                                    <div class="stat-mini-icon team">
                                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
                                            <circle cx="9" cy="7" r="4"/>
                                            <path d="M23 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75"/>
                                        </svg>
                                    </div>
                                    <div class="stat-mini-content">
                                        <span class="stat-mini-value">{team_stats().total}</span>
                                        <span class="stat-mini-label">"Team Tasks"</span>
                                    </div>
                                </div>
                            </div>

                            <div class="dashboard-grid-layout">
                                <div class="dashboard-main-column">
                                    <div class="section-header">
                                        <h2 class="section-title">"Recent Tasks"</h2>
                                        <Button variant=ButtonVariant::Ghost size=ButtonSize::Sm on_click=nav_tasks>
                                            "View all"
                                        </Button>
                                    </div>
                                    <div class="recent-tasks-grid">
                                        {let all_tasks: Vec<_> = recent_personal_tasks().into_iter().chain(recent_team_tasks().into_iter()).take(6).collect();
                                        if all_tasks.is_empty() {
                                            view! {
                                                <div class="empty-tasks">
                                                    <div class="empty-icon">
                                                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                                                            <path d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2"/>
                                                            <rect x="9" y="3" width="6" height="4" rx="1"/>
                                                        </svg>
                                                    </div>
                                                    <p>"No tasks yet"</p>
                                                    <Button variant=ButtonVariant::Primary size=ButtonSize::Sm on_click=nav_tasks>
                                                        "Create your first task"
                                                    </Button>
                                                </div>
                                            }.into_any()
                                        } else {
                                            all_tasks.into_iter().map(|t| {
                                                view! {
                                                    <div class="task-item-wrapper">
                                                        <TaskCard task=t variant=TaskCardVariant::Compact interactive=true />
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>().into_any()
                                        }}
                                    </div>
                                </div>

                                <div class="dashboard-sidebar">
                                    <Card>
                                        <div class="ws-card">
                                            <div class="ws-card-header">
                                                <h3 class="ws-card-title">"Live Activity"</h3>
                                                <div class=move || {
                                                    let status = ws_status.get();
                                                    let status_lower = status.to_lowercase();
                                                    let status_class = if status_lower.contains("connected") {
                                                        "ws-indicator connected"
                                                    } else if status_lower.contains("connecting") {
                                                        "ws-indicator connecting"
                                                    } else if status_lower.contains("error") {
                                                        "ws-indicator error"
                                                    } else {
                                                        "ws-indicator"
                                                    };
                                                    status_class
                                                }></div>
                                            </div>
                                            <p class="ws-subtitle">
                                                {ws_count.get()} events received
                                            </p>
                                            <div class="ws-logs-compact">
                                                {move || {
                                                    let logs = ws_logs.get();
                                                    if logs.is_empty() {
                                                        view! { <p class="ws-empty">"Waiting for updates..."</p> }.into_any()
                                                    } else {
                                                        logs.into_iter().take(3).map(|msg| {
                                                            view! { <div class="ws-log-compact">{msg}</div> }
                                                        }).collect::<Vec<_>>().into_any()
                                                    }
                                                }}
                                            </div>
                                            <CardFooter>
                                                <Button variant=ButtonVariant::Secondary size=ButtonSize::Sm on_click=refresh_button>
                                                    "Refresh"
                                                </Button>
                                            </CardFooter>
                                        </div>
                                    </Card>

                                    <Card>
                                        <div class="quick-actions">
                                            <h3 class="quick-actions-title">"Quick Actions"</h3>
                                            <div class="quick-actions-list">
                                                <button class="quick-action-btn" type="button" on:click=move |ev: leptos::ev::MouseEvent| { nav_tasks_clone.run((ev,)); }>
                                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                        <path d="M12 5v14M5 12h14"/>
                                                    </svg>
                                                    "New Task"
                                                </button>
                                                <button class="quick-action-btn" type="button" on:click=move |ev: leptos::ev::MouseEvent| { nav_teams_clone.run((ev,)); }>
                                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                        <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
                                                        <circle cx="9" cy="7" r="4"/>
                                                        <path d="M23 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75"/>
                                                    </svg>
                                                    "Join Team"
                                                </button>
                                                <button class="quick-action-btn" type="button" on:click=move |ev: leptos::ev::MouseEvent| {
                                                    let nav = use_navigate();
                                                    nav("/settings", Default::default());
                                                }>
                                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                        <circle cx="12" cy="12" r="3"/>
                                                        <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06a1.65 1.65 0 001.82.33H9a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"/>
                                                    </svg>
                                                    "Settings"
                                                </button>
                                            </div>
                                        </div>
                                    </Card>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}