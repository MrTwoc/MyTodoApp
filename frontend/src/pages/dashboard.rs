use crate::api::dashboard::get_overview;
use crate::api::ws::{connect_ws, WsConnection, WsEvent, WsState};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::card::{Card, CardFooter};
use crate::components::loading::{Loading, LoadingVariant};
use crate::components::task_card::TaskCard;
use crate::store::{use_api_client, use_user_store};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use serde_json::Value;

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
    let (ws_conn, set_ws_conn) = signal(Option::<WsConnection>::None);
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

    let load_dashboard = {
        let client = client.clone();
        Callback::from(move |_| {
            set_error.set(None);
            set_loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match get_overview(&client).await {
                    Ok(result) => {
                        set_overview.set(Some(result));
                    }
                    Err(err) => {
                        set_error.set(Some(err.message));
                    }
                }
                set_loading.set(false);
            });
        })
    };

    let connect_realtime = {
        let client = client.clone();
        let set_ws_status = set_ws_status;
        let set_ws_count = set_ws_count;
        let set_ws_logs = set_ws_logs;
        let ws_conn = ws_conn;
        let set_ws_conn = set_ws_conn;
        Callback::from(move |_| {
            if ws_conn.get_untracked().is_some() {
                return;
            }

            let on_state = Callback::from(move |state: WsState| match state {
                WsState::Connecting => set_ws_status.set("Connecting".to_string()),
                WsState::Open => set_ws_status.set("Connected".to_string()),
                WsState::Closed => set_ws_status.set("Closed".to_string()),
                WsState::Error(msg) => set_ws_status.set(format!("Error: {msg}")),
            });

            let on_message = Callback::from(move |evt: WsEvent| {
                let line = if evt.event.is_empty() {
                    format!("raw: {}", task_payload_to_text(&evt.payload))
                } else {
                    format!("{}: {}", evt.event, task_payload_to_text(&evt.payload))
                };

                set_ws_logs.update(|items| {
                    items.insert(0, line);
                    if items.len() > 5 {
                        items.truncate(5);
                    }
                });
                set_ws_count.update(|value| *value = value.saturating_add(1));
            });

            match connect_ws(&client, on_state, on_message) {
                Ok(conn) => {
                    set_ws_conn.set(Some(conn));
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
        move |_| n("/tasks", Default::default())
    };
    let nav_teams = {
        let n = navigate.clone();
        move |_| n("/teams", Default::default())
    };
    let nav_settings = {
        let n = navigate.clone();
        move |_| n("/settings", Default::default())
    };

    Effect::new(move |_| {
        if !initialized.get() {
            set_initialized.set(true);
            load_dashboard.run(());
            connect_realtime.run(());
            on_cleanup(move || {
                if let Some(conn) = ws_conn.get_untracked() {
                    conn.close();
                }
                set_ws_conn.set(None);
            });
        }
    });

    view! {
        <div class="dashboard">
            <header class="dashboard-header">
                <div>
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

            <Card title="Real-time Feed".to_string() subtitle="WebSocket notifications".to_string()>
                <div class="dashboard-ws">
                    <p class="dashboard-ws-status">{move || format!("Status: {}", ws_status.get())}</p>
                    <p class="dashboard-ws-status">{move || format!("Events: {}", ws_count.get())}</p>
                    <div class="dashboard-ws-log">
                        {move || {
                            let logs = ws_logs.get();
                            if logs.is_empty() {
                                view! { <p class="empty-text">"No events yet."</p> }.into_any()
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
                <Card title="Personal Task Board".to_string() subtitle="Current status".to_string()>
                    <div class="stat-row">
                        <div class="stat">
                            <span class="stat-number">{move || personal_stats().total}</span>
                            <span class="stat-label">{move || stat_label(personal_stats().total)}</span>
                        </div>
                        <div class="stat">
                            <span class="stat-number">{move || personal_stats().active}</span>
                            <span class="stat-label">"Active"</span>
                        </div>
                        <div class="stat">
                            <span class="stat-number">{move || personal_stats().completed}</span>
                            <span class="stat-label">"Completed"</span>
                        </div>
                        <div class="stat">
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

                <Card title="Team Task Board".to_string() subtitle="Current status".to_string()>
                    <div class="stat-row">
                        <div class="stat">
                            <span class="stat-number">{move || team_stats().total}</span>
                            <span class="stat-label">{move || stat_label(team_stats().total)}</span>
                        </div>
                        <div class="stat">
                            <span class="stat-number">{move || team_stats().active}</span>
                            <span class="stat-label">"Active"</span>
                        </div>
                        <div class="stat">
                            <span class="stat-number">{move || team_stats().completed}</span>
                            <span class="stat-label">"Completed"</span>
                        </div>
                        <div class="stat">
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

            <Show when=move || !loading.get() && error.get().is_none() fallback=|| {
                view! { <Loading variant=LoadingVariant::Spinner label="Loading dashboard...".to_string() /> }
            }>
                <div class="dashboard-sections">
                    <Show
                        when=move || !overview.get().is_none()
                        fallback=|| {
                            view! {
                                <Card title="No Data".to_string() subtitle="Nothing loaded yet".to_string()>
                                    <p class="empty-text">"Create a task first, then you can view it here."</p>
                                </Card>
                            }
                        }
                    >
                        <div class="dashboard-grid">
                            {move || {
                                let personal = overview
                                    .get()
                                    .map(|data| data.recent_personal_tasks)
                                    .unwrap_or_default();
                                let team = overview.get().map(|data| data.recent_team_tasks).unwrap_or_default();

                                if personal.is_empty() && team.is_empty() {
                                    view! {
                                        <Card title="Recent Task".to_string() subtitle="No recent tasks".to_string()>
                                            <p class="empty-text">"No recent tasks in the last period."</p>
                                        </Card>
                                    }.into_any()
                                } else {
                                    let mut cards = Vec::new();
                                    if !personal.is_empty() {
                                        let personal_cards = personal
                                            .into_iter()
                                            .map(|task| {
                                                let id = task.task_id;
                                                let nav = navigate.clone();
                                                view! {
                                                    <TaskCard
                                                        task=task
                                                        interactive=true
                                                        on_click=Some(Callback::from(move |_| {
                                                            nav(&format!("/tasks/{}", id), Default::default());
                                                        }))
                                                    />
                                                }
                                            })
                                            .collect::<Vec<_>>();
                                        let personal_card = view! {
                                            <div>
                                                <h3 class="task-list-title">"Recent Personal Tasks"</h3>
                                                <div class="task-grid">{personal_cards}</div>
                                            </div>
                                        };
                                        cards.push(personal_card.into_any());
                                    }

                                    if !team.is_empty() {
                                        let team_cards = team
                                            .into_iter()
                                            .map(|task| {
                                                let id = task.task_id;
                                                let nav = navigate.clone();
                                                view! {
                                                    <TaskCard
                                                        task=task
                                                        interactive=true
                                                        on_click=Some(Callback::from(move |_| {
                                                            nav(&format!("/tasks/{}", id), Default::default());
                                                        }))
                                                    />
                                                }
                                            })
                                            .collect::<Vec<_>>();
                                        let team_card = view! {
                                            <div>
                                                <h3 class="task-list-title">"Recent Team Tasks"</h3>
                                                <div class="task-grid">{team_cards}</div>
                                            </div>
                                        };
                                        cards.push(team_card.into_any());
                                    }

                                    view! { <div class="dashboard-recent-list">{cards}</div> }.into_any()
                                }
                            }}
                        </div>
                    </Show>

                    <Show when=move || error.get().is_some() fallback=|| ().into_any()>
                        {move || {
                            if let Some(err) = error.get() {
                                view! {
                                    <Card title="Load failed".to_string() subtitle="Dashboard data load error".to_string()>
                                        <p class="auth-error">{err}</p>
                                    </Card>
                                }.into_any()
                            } else {
                                ().into_any()
                            }
                        }}
                    </Show>
                </div>
            </Show>
        </div>
    }
}
