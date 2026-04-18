use crate::api::dashboard::get_overview;
use crate::api::ws::{WsConnection, WsEvent, WsState, connect_ws};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::loading::{Loading, LoadingVariant};
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

fn get_greeting() -> &'static str {
    use chrono::Timelike;
    let h = chrono::Local::now().hour();
    if h < 6 {
        "夜深了"
    } else if h < 12 {
        "早上好"
    } else if h < 14 {
        "中午好"
    } else if h < 18 {
        "下午好"
    } else {
        "晚上好"
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

    let active_tasks = move || {
        let p = personal_stats();
        let t = team_stats();
        p.active + t.active
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

    let completion_rate = move || {
        let total = total_tasks();
        if total == 0 {
            0_u32
        } else {
            let completed = completed_tasks();
            (completed * 100 / total).min(100)
        }
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

            match connect_ws(
                &client,
                Callback::new(move |(state,): (WsState,)| {
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                        match state {
                            WsState::Connecting => set_ws_status_inner.set("Connecting".to_string()),
                            WsState::Open => set_ws_status_inner.set("Connected".to_string()),
                            WsState::Closed => set_ws_status_inner.set("Closed".to_string()),
                            WsState::Error(msg) => {
                                set_ws_status_inner.set(format!("Error: {msg}"))
                            }
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
                }),
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

    // Ring SVG for completion rate
    let completion_ring_svg = move || {
        let rate = completion_rate();
        let circumference = 2.0 * std::f64::consts::PI * 36.0;
        let offset = circumference * (1.0 - rate as f64 / 100.0);
        format!(
            r#"<svg class="stat-ring" viewBox="0 0 88 88"><circle cx="44" cy="44" r="36" fill="none" stroke="var(--border-primary)" stroke-width="6"/><circle cx="44" cy="44" r="36" fill="none" stroke="var(--accent-primary)" stroke-width="6" stroke-dasharray="{}" stroke-dashoffset="{}" stroke-linecap="round" transform="rotate(-90 44 44)"/></svg>"#,
            circumference, offset
        )
    };

    view! {
        <div class="dashboard-v3">
            // Background glow orbs
            <div class="db-glow">
                <div class="db-glow-orb glow-1"></div>
                <div class="db-glow-orb glow-2"></div>
                <div class="db-glow-orb glow-3"></div>
            </div>

            {move || {
                if loading.get() {
                    view! {
                        <div class="db-loading">
                            <Loading variant=LoadingVariant::Spinner label="Loading...".to_string() size=40 />
                        </div>
                    }.into_any()
                } else if let Some(err) = error.get() {
                    view! {
                        <div class="db-error">
                            <div class="db-error-icon">
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <circle cx="12" cy="12" r="10"/>
                                    <path d="M12 8v4M12 16h.01"/>
                                </svg>
                            </div>
                            <p class="db-error-text">{err}</p>
                            <Button variant=ButtonVariant::Primary size=ButtonSize::Md on_click=refresh_button>
                                "重试"
                            </Button>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="db-content">
                            // Greeting Area
                            <header class="db-greeting">
                                <div class="db-greeting-left">
                                    <h1 class="db-greeting-title">
                                        {get_greeting()} "，" {user_label()}
                                    </h1>
                                    <p class="db-greeting-sub">
                                        "今天有 "
                                        <span class="db-accent-text">{active_tasks()}</span>
                                        " 件任务进行中，已完成 "
                                        <span class="db-accent-text">{completed_tasks()}</span>
                                        " 项"
                                    </p>
                                </div>
                                <div class="db-greeting-right">
                                    <Button variant=ButtonVariant::Primary size=ButtonSize::Md on_click=nav_tasks>
                                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="width:16px;height:16px">
                                            <path d="M12 5v14M5 12h14"/>
                                        </svg>
                                        " 新建任务"
                                    </Button>
                                    <Button variant=ButtonVariant::Secondary size=ButtonSize::Md on_click=nav_teams>
                                        "团队"
                                    </Button>
                                </div>
                            </header>

                            // KPI Stats Bar
                            <div class="db-stats-bar">
                                <div class="db-stat-card">
                                    <div class="db-stat-icon stat-icon-todo">
                                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                            <circle cx="12" cy="12" r="10"/>
                                            <path d="M12 6v6l4 2"/>
                                        </svg>
                                    </div>
                                    <div class="db-stat-info">
                                        <span class="db-stat-label">"进行中"</span>
                                        <span class="db-stat-num">{active_tasks()}</span>
                                    </div>
                                    <div class="db-stat-ring-wrap" inner_html=completion_ring_svg()></div>
                                </div>

                                <div class="db-stat-card">
                                    <div class="db-stat-icon stat-icon-doing">
                                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                            <path d="M22 11.08V12a10 10 0 11-5.93-9.14"/>
                                            <path d="M22 4L12 14.01l-3-3"/>
                                        </svg>
                                    </div>
                                    <div class="db-stat-info">
                                        <span class="db-stat-label">"已完成"</span>
                                        <span class="db-stat-num">{completed_tasks()}</span>
                                    </div>
                                </div>

                                <div class="db-stat-card">
                                    <div class="db-stat-icon stat-icon-personal">
                                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                                            <circle cx="12" cy="7" r="4"/>
                                        </svg>
                                    </div>
                                    <div class="db-stat-info">
                                        <span class="db-stat-label">"个人任务"</span>
                                        <span class="db-stat-num">{personal_stats().total}</span>
                                    </div>
                                </div>

                                <div class="db-stat-card">
                                    <div class="db-stat-icon stat-icon-team">
                                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
                                            <circle cx="9" cy="7" r="4"/>
                                            <path d="M23 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75"/>
                                        </svg>
                                    </div>
                                    <div class="db-stat-info">
                                        <span class="db-stat-label">"团队任务"</span>
                                        <span class="db-stat-num">{team_stats().total}</span>
                                    </div>
                                </div>
                            </div>

                            // Main Grid: Tasks + Sidebar
                            <div class="db-main-grid">
                                // Left Column: Recent Tasks
                                <div class="db-section-card">
                                    <div class="db-section-header">
                                        <div class="db-section-title">
                                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="width:18px;height:18px">
                                                <path d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2"/>
                                                <rect x="9" y="3" width="6" height="4" rx="1"/>
                                            </svg>
                                            " 近期待办"
                                        </div>
                                        <span class="db-section-badge db-badge-orange">
                                            {move || {
                                                let p = recent_personal_tasks();
                                                let t = recent_team_tasks();
                                                (p.len() + t.len()).to_string()
                                            }}
                                        </span>
                                    </div>
                                    <div class="db-section-body">
                                        {let all_tasks: Vec<_> = recent_personal_tasks().into_iter().chain(recent_team_tasks().into_iter()).take(6).collect();
                                        if all_tasks.is_empty() {
                                            view! {
                                                <div class="db-empty-tasks">
                                                    <div class="db-empty-icon">
                                                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                                                            <path d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2"/>
                                                            <rect x="9" y="3" width="6" height="4" rx="1"/>
                                                        </svg>
                                                    </div>
                                                    <p>"暂无任务"</p>
                                                    <Button variant=ButtonVariant::Primary size=ButtonSize::Sm on_click=nav_tasks_clone>
                                                        "创建第一个任务"
                                                    </Button>
                                                </div>
                                            }.into_any()
                                        } else {
                                            all_tasks.into_iter().map(|t| {
                                                let status_label = match t.task_status {
                                                    crate::store::task_store::TaskStatus::Active => "进行中",
                                                    crate::store::task_store::TaskStatus::Completed => "已完成",
                                                    crate::store::task_store::TaskStatus::Paused => "已暂停",
                                                };
                                                let status_cls = match t.task_status {
                                                    crate::store::task_store::TaskStatus::Active => "task-status-active",
                                                    crate::store::task_store::TaskStatus::Completed => "task-status-completed",
                                                    crate::store::task_store::TaskStatus::Paused => "task-status-paused",
                                                };
                                                let priority_cls = match t.task_priority {
                                                    0..=2 => "priority-low",
                                                    3..=5 => "priority-medium",
                                                    _ => "priority-high",
                                                };
                                                let priority_label = match t.task_priority {
                                                    0..=2 => "低",
                                                    3..=5 => "中",
                                                    _ => "高",
                                                };
                                                view! {
                                                    <div class=format!("db-task-item {}", priority_cls)>
                                                        <div class="db-task-priority-bar"></div>
                                                        <div class="db-task-info">
                                                            <div class="db-task-title">{t.task_name}</div>
                                                            <div class="db-task-meta">
                                                                <span class=format!("db-task-tag {}", status_cls)>{status_label}</span>
                                                                <span class=format!("db-task-tag {}", priority_cls)>{priority_label}</span>
                                                                <span class="db-task-diff">{"难度: "}{t.task_difficulty}</span>
                                                            </div>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>().into_any()
                                        }}
                                    </div>
                                </div>

                                // Right Column: Live Activity + Quick Actions
                                <div class="db-sidebar-stack">
                                    // Live Activity Card
                                    <div class="db-section-card">
                                        <div class="db-section-header">
                                            <div class="db-section-title">
                                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="width:18px;height:18px">
                                                    <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/>
                                                </svg>
                                                " 实时动态"
                                            </div>
                                            <div class=move || {
                                                let status = ws_status.get();
                                                let status_lower = status.to_lowercase();
                                                if status_lower.contains("connected") {
                                                    "db-ws-dot db-ws-connected".to_string()
                                                } else if status_lower.contains("connecting") {
                                                    "db-ws-dot db-ws-connecting".to_string()
                                                } else if status_lower.contains("error") {
                                                    "db-ws-dot db-ws-error".to_string()
                                                } else {
                                                    "db-ws-dot".to_string()
                                                }
                                            }></div>
                                        </div>
                                        <div class="db-section-body">
                                            <p class="db-ws-count-text">
                                                {ws_count.get()} " 条实时消息"
                                            </p>
                                            <div class="db-ws-logs">
                                                {move || {
                                                    let logs = ws_logs.get();
                                                    if logs.is_empty() {
                                                        view! { <p class="db-ws-empty">"等待实时更新..."</p> }.into_any()
                                                    } else {
                                                        logs.into_iter().take(4).map(|msg| {
                                                            view! { <div class="db-ws-log-item">{msg}</div> }
                                                        }).collect::<Vec<_>>().into_any()
                                                    }
                                                }}
                                            </div>
                                        </div>
                                    </div>

                                    // Quick Actions Card
                                    <div class="db-section-card">
                                        <div class="db-section-header">
                                            <div class="db-section-title">
                                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="width:18px;height:18px">
                                                    <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/>
                                                </svg>
                                                " 快捷操作"
                                            </div>
                                        </div>
                                        <div class="db-section-body">
                                            <div class="db-quick-actions">
                                                <button class="db-quick-btn" type="button" on:click=move |ev: leptos::ev::MouseEvent| { nav_tasks.run((ev,)); }>
                                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                        <path d="M12 5v14M5 12h14"/>
                                                    </svg>
                                                    "新建任务"
                                                </button>
                                                <button class="db-quick-btn" type="button" on:click=move |ev: leptos::ev::MouseEvent| { nav_teams.run((ev,)); }>
                                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                        <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
                                                        <circle cx="9" cy="7" r="4"/>
                                                        <path d="M23 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75"/>
                                                    </svg>
                                                    "加入团队"
                                                </button>
                                                <button class="db-quick-btn" type="button" on:click=move |_| {
                                                    let nav = use_navigate();
                                                    nav("/settings", Default::default());
                                                }>
                                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                        <circle cx="12" cy="12" r="3"/>
                                                        <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06a1.65 1.65 0 001.82.33H9a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"/>
                                                    </svg>
                                                    "设置"
                                                </button>
                                            </div>
                                        </div>
                                    </div>

                                    // Completion Summary Card
                                    <div class="db-section-card">
                                        <div class="db-section-header">
                                            <div class="db-section-title">
                                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="width:18px;height:18px">
                                                    <path d="M18 20V10M12 20V4M6 20v-6"/>
                                                </svg>
                                                " 任务概览"
                                            </div>
                                        </div>
                                        <div class="db-section-body">
                                            <div class="db-overview-bar">
                                                <div class="db-overview-label">
                                                    <span>"完成率"</span>
                                                    <span class="db-overview-pct">{completion_rate()}"%"</span>
                                                </div>
                                                <div class="db-overview-track">
                                                    <div class="db-overview-fill" style=format!("width: {}%", completion_rate())></div>
                                                </div>
                                            </div>
                                            <div class="db-overview-stats">
                                                <div class="db-overview-stat">
                                                    <span class="db-overview-stat-num">{total_tasks()}</span>
                                                    <span class="db-overview-stat-label">"总任务"</span>
                                                </div>
                                                <div class="db-overview-stat">
                                                    <span class="db-overview-stat-num">{active_tasks()}</span>
                                                    <span class="db-overview-stat-label">"进行中"</span>
                                                </div>
                                                <div class="db-overview-stat">
                                                    <span class="db-overview-stat-num">{completed_tasks()}</span>
                                                    <span class="db-overview-stat-label">"已完成"</span>
                                                </div>
                                                <div class="db-overview-stat">
                                                    <span class="db-overview-stat-num">{personal_stats().paused + team_stats().paused}</span>
                                                    <span class="db-overview-stat-label">"已暂停"</span>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
