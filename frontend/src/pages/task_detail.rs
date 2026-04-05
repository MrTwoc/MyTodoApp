use crate::api::task::delete_task as api_delete_task;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::card::Card;
use crate::components::modal::Modal;
use crate::components::task_form::{TaskFormData, TaskFormModal, TaskFormMode};
use crate::store::task_store::{Task, TaskStatus};
use crate::store::{use_api_client, use_task_store};
use chrono::Utc;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn format_timestamp(ts: i64) -> String {
    let ms = (ts * 1000) as f64;
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ms));
    let year = date.get_full_year();
    let month = date.get_month() + 1;
    let day = date.get_date();
    format!("{:04}/{:02}/{:02}", year, month, day)
}

fn priority_label(p: u8) -> &'static str {
    match p {
        0..=2 => "Low",
        3..=5 => "Medium",
        6..=8 => "High",
        _ => "Urgent",
    }
}

fn priority_class(p: u8) -> &'static str {
    match p {
        0..=2 => "priority-low",
        3..=5 => "priority-medium",
        6..=8 => "priority-high",
        _ => "priority-urgent",
    }
}

fn status_label(s: &TaskStatus) -> &'static str {
    match s {
        TaskStatus::Active => "Active",
        TaskStatus::Completed => "Completed",
        TaskStatus::Paused => "Paused",
    }
}

fn status_class(s: &TaskStatus) -> &'static str {
    match s {
        TaskStatus::Active => "status-active",
        TaskStatus::Completed => "status-completed",
        TaskStatus::Paused => "status-paused",
    }
}

fn status_progress(s: &TaskStatus) -> u8 {
    match s {
        TaskStatus::Active => 33,
        TaskStatus::Paused => 50,
        TaskStatus::Completed => 100,
    }
}

// ── Mock data ─────────────────────────────────────────────────────────────────

fn mock_task(id: u64) -> Task {
    use std::collections::HashSet;
    let mut keywords = HashSet::new();
    keywords.insert("example".to_string());
    keywords.insert("mock".to_string());

    Task {
        task_id: id,
        task_name: format!("Sample Task #{}", id),
        task_description: Some(
            "This is a placeholder task. Wire up the API to load real data.".to_string(),
        ),
        task_keywords: keywords,
        task_priority: 3,
        task_deadline: Some(1_800_000_000),
        task_status: TaskStatus::Active,
        task_create_time: 1_700_000_000,
        task_leader_id: 1,
        task_team_id: None,
        task_update_time: None,
        task_complete_time: None,
    }
}

// ── TaskDetailPage ────────────────────────────────────────────────────────────

#[component]
pub fn TaskDetailPage() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();

    let task_id: u64 = params
        .get()
        .get("task_id")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let (task, set_task) = signal(mock_task(task_id));
    let (show_edit_modal, set_show_edit_modal) = signal(false);
    let client = use_api_client();
    let task_store = use_task_store();
    // let offline_store = use_offline_task_store();

    // let is_offline_mode = move || offline_store.state.get().enabled;

    let current_task = {
        // let offline_store = offline_store.clone();
        let task_signal = task.clone();
        let task_id = task_id;
        // let is_offline = is_offline_mode.clone();
        move || {
            // if is_offline() {
            //     let tasks = offline_store.state.get().tasks;
            //     let offline_task = tasks.into_iter().find(|t| t.task_id == task_id);
            //     if let Some(t) = offline_task {
            //         return t;
            //     }
            // }
            task_signal.get()
        }
    };

    let nav_back = {
        let n = navigate.clone();
        move |_| n("/tasks", Default::default())
    };

    // ── Status switch ─────────────────────────────────────────────────────────
    let switch_to = {
        let task = task.clone();
        let set_task = set_task.clone();
        // let offline_store = offline_store.clone();
        // let is_offline = is_offline_mode.clone();
        let task_id = task_id;
        move |new_status: TaskStatus| {
            // if is_offline() {
            //     offline_store.set_task_status(task_id, new_status);
            // } else {
                let mut t = task.get();
                t.task_status = new_status;
                set_task.set(t);
            // }
        }
    };

    let switch_active = {
        let switch_to = switch_to.clone();
        move |_: web_sys::MouseEvent| switch_to(TaskStatus::Active)
    };
    let switch_completed = {
        let switch_to = switch_to.clone();
        move |_: web_sys::MouseEvent| switch_to(TaskStatus::Completed)
    };
    let switch_paused = {
        let switch_to = switch_to.clone();
        move |_: web_sys::MouseEvent| switch_to(TaskStatus::Paused)
    };

    // ── Edit form callbacks ───────────────────────────────────────────────────
    let on_edit_submit: Callback<(TaskFormData,), ()> = {
        let task = task.clone();
        let set_task = set_task.clone();
        // let offline_store = offline_store.clone();
        // let is_offline = is_offline_mode.clone();
        let task_id = task_id;
        Callback::from(move |data: TaskFormData| {
            // if is_offline() {
            //     let mut state = offline_store.state.get();
            //     if let Some(t) = state.tasks.iter_mut().find(|t| t.task_id == task_id) {
            //         t.task_name = data.task_name;
            //         t.task_description = data.task_description;
            //         t.task_keywords = data.task_keywords.into_iter().collect();
            //         t.task_priority = data.task_priority;
            //         t.task_deadline = data.task_deadline;
            //         t.task_update_time = Some(Utc::now().timestamp());
            //         offline_store.set_state.set(state);
            //     }
            // } else {
                let mut t = task.get();
                t.task_name = data.task_name;
                t.task_description = data.task_description;
                t.task_keywords = data.task_keywords.into_iter().collect();
                t.task_priority = data.task_priority;
                t.task_deadline = data.task_deadline;
                set_task.set(t);
            // }
            set_show_edit_modal.set(false);
        })
    };

    let on_edit_cancel: Callback<(), ()> = Callback::from(move || {
        set_show_edit_modal.set(false);
    });

    // ── Delete handler ────────────────────────────────────────────────────────
    let on_delete = {
        let navigate = navigate.clone();
        let client = client.clone();
        let task_store = task_store.clone();
        // let offline_store = offline_store.clone();
        let task_id = task_id;
        // let is_offline = is_offline_mode.clone();
        Callback::from(move |_: web_sys::MouseEvent| {
            // if is_offline() {
            //     offline_store.delete_task(task_id);
            //     navigate("/tasks", Default::default());
            // } else {
                let client = client.clone();
                let task_store = task_store.clone();
                let navigate = navigate.clone();
                let task_id = task_id;
                wasm_bindgen_futures::spawn_local(async move {
                    match api_delete_task(&client, task_id).await {
                        Ok(_) => {
                            task_store.remove_task(task_id);
                            navigate("/tasks", Default::default());
                        }
                        Err(e) => {
                            tracing::error!("Failed to delete task: {}", e.message);
                        }
                    }
                });
            // }
        })
    };

    view! {
        <div class="page">
            // ── Header ────────────────────────────────────────────────────────
            <header class="page-header task-detail-header">
                <div>
                    <button class="back-btn" on:click=nav_back>"← Back"</button>
                    <h1 class="page-title task-detail-title">{move || current_task().task_name.clone()}</h1>
                </div>
                <div class="task-detail-actions">
                    <Button
                        variant=ButtonVariant::Secondary
                        size=ButtonSize::Sm
                        on_click=Callback::from(move |_: web_sys::MouseEvent| {
                            set_show_edit_modal.set(true);
                        })
                    >
                        "Edit"
                    </Button>
                    <Button
                        variant=ButtonVariant::Danger
                        size=ButtonSize::Sm
                        on_click=on_delete
                    >
                        "Delete"
                    </Button>
                </div>
            </header>

            // ── Status & Priority badges ──────────────────────────────────────
            <div class="task-detail-badges">
                {move || {
                    let t = current_task();
                    let sc = status_class(&t.task_status);
                    let sl = status_label(&t.task_status);
                    let pc = priority_class(t.task_priority);
                    let pl = priority_label(t.task_priority);
                    view! {
                        <span class=format!("task-status-badge {}", sc)>{sl}</span>
                        <span class=format!("task-priority-badge {}", pc)>{pl}</span>
                    }
                }}
            </div>

            // ── Progress bar ──────────────────────────────────────────────────
            <div class="task-detail-progress">
                <div class="task-detail-progress-label">
                    <span>"Progress"</span>
                    <span class="task-detail-progress-value">{move || {
                        let pct = status_progress(&current_task().task_status);
                        format!("{}%", pct)
                    }}</span>
                </div>
                <div class="progress-bar">
                    {move || {
                        let pct = status_progress(&current_task().task_status);
                        view! {
                            <div
                                class="progress-fill"
                                style=format!("width: {}%;", pct)
                            />
                        }
                    }}
                </div>
            </div>

            // ── Status switch ─────────────────────────────────────────────────
            <Card title="Status".to_string()>
                <div class="status-switch">
                    {move || {
                        let current = current_task().task_status.clone();
                        let st_active = switch_to.clone();
                        let st_completed = switch_to.clone();
                        let st_paused = switch_to.clone();
                        view! {
                            <Button
                                variant=if current == TaskStatus::Active { ButtonVariant::Primary } else { ButtonVariant::Secondary }
                                size=ButtonSize::Sm
                                on_click=Callback::from(move |_: web_sys::MouseEvent| st_active(TaskStatus::Active))
                            >
                                "Active"
                            </Button>
                            <Button
                                variant=if current == TaskStatus::Completed { ButtonVariant::Primary } else { ButtonVariant::Secondary }
                                size=ButtonSize::Sm
                                on_click=Callback::from(move |_: web_sys::MouseEvent| st_completed(TaskStatus::Completed))
                            >
                                "Completed"
                            </Button>
                            <Button
                                variant=if current == TaskStatus::Paused { ButtonVariant::Primary } else { ButtonVariant::Secondary }
                                size=ButtonSize::Sm
                                on_click=Callback::from(move |_: web_sys::MouseEvent| st_paused(TaskStatus::Paused))
                            >
                                "Paused"
                            </Button>
                        }
                    }}
                </div>
            </Card>

            // ── Details card ──────────────────────────────────────────────────
            <Card title="Details".to_string()>
                {move || {
                    current_task().task_description.map(|desc| view! {
                        <div class="task-detail-section">
                            <h4 class="task-detail-label">"Description"</h4>
                            <p class="task-detail-value">{desc}</p>
                        </div>
                    })
                }}
                {move || {
                    current_task().task_deadline.map(|ts| {
                        let s = format_timestamp(ts);
                        view! {
                            <div class="task-detail-section">
                                <h4 class="task-detail-label">"Deadline"</h4>
                                <p class="task-detail-value">{s}</p>
                            </div>
                        }
                    })
                }}
                {move || {
                    let kws: Vec<String> = current_task().task_keywords.into_iter().collect();
                    if kws.is_empty() {
                        ().into_any()
                    } else {
                        view! {
                            <div class="task-detail-section">
                                <h4 class="task-detail-label">"Tags"</h4>
                                <div class="tag-chips">
                                    {kws.into_iter().map(|k| view! {
                                        <span class="tag-chip">{k}</span>
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        }.into_any()
                    }
                }}
            </Card>

            // ── History / timeline ────────────────────────────────────────────
            <Card title="History".to_string()>
                <p class="task-detail-empty">"No history available."</p>
            </Card>

            // ── Edit modal ────────────────────────────────────────────────────
            <TaskFormModal
                open=MaybeSignal::from(show_edit_modal)
                mode=TaskFormMode::Edit
                initial_data=TaskFormData::from(current_task())
                on_submit=on_edit_submit
                on_close=on_edit_cancel
            />
        </div>
    }
}
