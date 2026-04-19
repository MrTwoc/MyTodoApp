use crate::api::task::{delete_task as api_delete_task, get_task as api_get_task, get_task_logs as api_get_task_logs, update_task as api_update_task};
use crate::api::user::get_user as api_get_user;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::modal::Modal;
use crate::components::task_comment::TaskComments;
use crate::store::task_store::{Task, TaskStatus};
use crate::store::user_store::UserProfile;
use crate::store::{use_api_client, use_task_store};
use leptos::ev;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use wasm_bindgen::JsCast;

fn format_timestamp(ts: i64) -> String {
    let ms = (ts * 1000) as f64;
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ms));
    let year = date.get_full_year();
    let month = date.get_month() + 1;
    let day = date.get_date();
    format!("{:04}/{:02}/{:02}", year, month, day)
}

fn format_log_timestamp(ts: i64) -> String {
    let ms = (ts * 1000) as f64;
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ms));
    let year = date.get_full_year();
    let month = date.get_month() + 1;
    let day = date.get_date();
    let hours = date.get_hours();
    let minutes = date.get_minutes();
    format!("{:04}/{:02}/{:02} {:02}:{:02}", year, month, day, hours, minutes)
}

fn format_action_type(action: &str) -> String {
    if action.contains("Created") {
        "Created".to_string()
    } else if action.contains("Updated") {
        "Updated".to_string()
    } else if action.contains("Deleted") {
        "Deleted".to_string()
    } else if action.contains("StatusChanged") {
        "Status Changed".to_string()
    } else if action.contains("PriorityChanged") {
        "Priority Changed".to_string()
    } else if action.contains("DeadlineChanged") {
        "Deadline Changed".to_string()
    } else if action.contains("LeaderChanged") {
        "Leader Changed".to_string()
    } else if action.contains("TeamChanged") {
        "Team Changed".to_string()
    } else if action.contains("CommentAdded") {
        "Comment Added".to_string()
    } else if action.contains("AttachmentAdded") {
        "Attachment Added".to_string()
    } else {
        action.to_string()
    }
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

#[derive(Clone)]
struct EditableTaskData {
    task_name: String,
    task_description: Option<String>,
    task_priority: u8,
    task_difficulty: u8,
    task_deadline: Option<i64>,
    task_status: TaskStatus,
}

impl Default for EditableTaskData {
    fn default() -> Self {
        Self {
            task_name: String::new(),
            task_description: None,
            task_priority: 5,
            task_difficulty: 0,
            task_deadline: None,
            task_status: TaskStatus::Active,
        }
    }
}

impl From<Task> for EditableTaskData {
    fn from(task: Task) -> Self {
        Self {
            task_name: task.task_name,
            task_description: task.task_description,
            task_priority: task.task_priority,
            task_difficulty: task.task_difficulty,
            task_deadline: task.task_deadline,
            task_status: task.task_status,
        }
    }
}

fn event_target_value(ev: &ev::Event) -> String {
    ev.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>().value()
}

#[component]
pub fn TaskDetailPage() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();

    let task_id: u64 = params
        .get()
        .get("task_id")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let client = use_api_client();
    let task_store = use_task_store();

    let is_loading = RwSignal::new(true);
    let is_editing = RwSignal::new(false);
    let is_saving = RwSignal::new(false);
    let save_error = RwSignal::new(Option::<String>::None);
    let show_delete_confirm = RwSignal::new(false);

    let (task, set_task) = signal(Task::default());
    let (creator, set_creator) = signal(Option::<UserProfile>::None);
    let edit_data = RwSignal::new(EditableTaskData::default());
    let (task_logs, set_task_logs) = signal(Vec::<serde_json::Value>::new());
    let (operator_names, set_operator_names) = signal(std::collections::HashMap::<u64, String>::new());

    let load_task = {
        let client = client.clone();
        let task_id = task_id;
        let set_task = set_task.clone();
        let set_creator = set_creator.clone();
        let set_task_logs = set_task_logs.clone();
        let set_operator_names = set_operator_names.clone();
        let edit_data = edit_data.clone();
        let is_loading = is_loading.clone();
        move || {
            let client = client.clone();
            let task_id = task_id;
            let set_task = set_task.clone();
            let set_creator = set_creator.clone();
            let set_task_logs = set_task_logs.clone();
            let set_operator_names = set_operator_names.clone();
            let edit_data = edit_data.clone();
            let is_loading = is_loading.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api_get_task(&client, task_id).await {
                    Ok(loaded_task) => {
                        set_task.set(loaded_task.clone());
                        edit_data.set(EditableTaskData::from(loaded_task.clone()));
                        
                        match api_get_user(&client, loaded_task.task_leader_id).await {
                            Ok(user) => {
                                set_creator.set(Some(user));
                            }
                            Err(e) => {
                                tracing::error!("Failed to load creator: {}", e.message);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to load task: {}", e.message);
                    }
                }

                match api_get_task_logs(&client, task_id).await {
                    Ok(logs) => {
                        set_task_logs.set(logs.clone());
                        
                        let mut unique_ops = std::collections::HashSet::new();
                        for log in &logs {
                            if let Some(op_id) = log.get("operator_id").and_then(|v| v.as_u64()) {
                                unique_ops.insert(op_id);
                            }
                        }
                        
                        let mut names = std::collections::HashMap::new();
                        for op_id in unique_ops {
                            match api_get_user(&client, op_id).await {
                                Ok(user) => {
                                    names.insert(op_id, user.username);
                                }
                                Err(_) => {
                                    names.insert(op_id, format!("User {}", op_id));
                                }
                            }
                        }
                        set_operator_names.set(names);
                    }
                    Err(e) => {
                        tracing::error!("Failed to load task logs: {}", e.message);
                    }
                }

                is_loading.set(false);
            });
        }
    };

    Effect::new(move |_| {
        load_task();
    });

    let current_task = {
        let task_signal = task.clone();
        move || task_signal.get()
    };

    let nav_back = {
        let n = navigate.clone();
        let task = task.clone();
        move |_| {
            let t = task.get();
            if let Some(team_id) = t.task_team_id {
                n(&format!("/teams/{}", team_id), Default::default())
            } else {
                n("/tasks", Default::default())
            }
        }
    };

    let start_edit = {
        let task = task.clone();
        let edit_data = edit_data.clone();
        Callback::from(move |_: web_sys::MouseEvent| {
            let t = task.get();
            edit_data.set(EditableTaskData::from(t));
            is_editing.set(true);
            save_error.set(None);
        })
    };

    let cancel_edit = Callback::from(move |_: web_sys::MouseEvent| {
        is_editing.set(false);
        save_error.set(None);
    });

    let update_edit_field = move |field: &str, value: String| {
        edit_data.update(|data| {
            match field {
                "task_name" => data.task_name = value,
                "task_description" => data.task_description = if value.is_empty() { None } else { Some(value) },
                "task_priority" => {
                    if let Ok(p) = value.parse::<u8>() {
                        data.task_priority = p.min(10);
                    }
                }
                "task_difficulty" => {
                    if let Ok(d) = value.parse::<u8>() {
                        data.task_difficulty = d.min(10);
                    }
                }
                "task_deadline" => {
                    if value.is_empty() {
                        data.task_deadline = None;
                    } else if let Ok(date) = chrono::NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
                        let timestamp = date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
                        data.task_deadline = Some(timestamp);
                    }
                }
                "task_status" => {
                    data.task_status = match value.as_str() {
                        "Active" => TaskStatus::Active,
                        "Completed" => TaskStatus::Completed,
                        "Paused" => TaskStatus::Paused,
                        _ => data.task_status.clone(),
                    };
                }
                _ => {}
            }
        });
    };

    let save_edit = {
        let task = task.clone();
        let set_task = set_task.clone();
        let edit_data = edit_data.clone();
        let client = client.clone();
        let task_id = task_id;
        let task_store = task_store.clone();
        Callback::from(move |_: web_sys::MouseEvent| {
            let data = edit_data.get();
            let client = client.clone();
            let task_id = task_id;
            let task = task.clone();
            let set_task = set_task.clone();
            let task_store = task_store.clone();
            let is_saving = is_saving.clone();
            let save_error = save_error.clone();
            let is_editing = is_editing.clone();

            is_saving.set(true);
            save_error.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                use crate::api::task::UpdateTaskRequest;

                let req = UpdateTaskRequest {
                    task_name: Some(data.task_name.clone()),
                    task_description: data.task_description.clone(),
                    task_keywords: None,
                    task_priority: Some(data.task_priority),
                    task_difficulty: Some(data.task_difficulty),
                    task_deadline: data.task_deadline,
                    task_status: Some(match data.task_status {
                        TaskStatus::Active => "Active".to_string(),
                        TaskStatus::Completed => "Completed".to_string(),
                        TaskStatus::Paused => "Paused".to_string(),
                    }),
                };

                match api_update_task(&client, task_id, &req).await {
                    Ok(updated_task) => {
                        set_task.set(updated_task.clone());
                        task_store.update_task(task_id, updated_task);
                        is_editing.set(false);
                    }
                    Err(e) => {
                        tracing::error!("Failed to update task: {}", e.message);
                        save_error.set(Some(e.message));
                    }
                }
                is_saving.set(false);
            });
        })
    };

    let on_delete = Callback::from(move |_: web_sys::MouseEvent| {
        show_delete_confirm.set(true);
    });

    let confirm_delete = {
        let navigate = navigate.clone();
        let client = client.clone();
        let task_store = task_store.clone();
        let task_id = task_id;
        let set_show_delete_confirm = show_delete_confirm.clone();
        Callback::from(move |_: web_sys::MouseEvent| {
            let client = client.clone();
            let task_store = task_store.clone();
            let navigate = navigate.clone();
            let task_id = task_id;
            let set_show_delete_confirm = set_show_delete_confirm.clone();
            set_show_delete_confirm.set(false);
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
        })
    };

    let cancel_delete = Callback::from(move |_: web_sys::MouseEvent| {
        show_delete_confirm.set(false);
    });

    view! {
        <div class="page task-detail-page">
            // Top Bar
            <header class="task-detail-topbar">
                <button class="back-btn" on:click=nav_back>
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M19 12H5M12 19l-7-7 7-7"/>
                    </svg>
                    "Back"
                </button>
                <div class="task-detail-topbar-actions">
                    {move || {
                        if is_editing.get() {
                            let saving = is_saving.get();
                            view! {
                                <Button
                                    variant=ButtonVariant::Secondary
                                    size=ButtonSize::Sm
                                    disabled=saving
                                    on_click=cancel_edit
                                >
                                    "Cancel"
                                </Button>
                                <Button
                                    variant=ButtonVariant::Primary
                                    size=ButtonSize::Sm
                                    disabled=saving
                                    on_click=save_edit
                                >
                                    {if saving { "Saving..." } else { "Save Changes" }}
                                </Button>
                            }
                        } else {
                            view! {
                                <Button
                                    variant=ButtonVariant::Danger
                                    size=ButtonSize::Sm
                                    on_click=on_delete
                                >
                                    "Delete"
                                </Button>
                                <Button
                                    variant=ButtonVariant::Primary
                                    size=ButtonSize::Sm
                                    on_click=start_edit
                                >
                                    "Edit Task"
                                </Button>
                            }
                        }
                    }}
                </div>
            </header>

            // Error Toast
            {move || {
                if let Some(err) = save_error.get() {
                    view! {
                        <div class="task-detail-error-toast">
                            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <circle cx="12" cy="12" r="10"/>
                                <line x1="15" y1="9" x2="9" y2="15"/>
                                <line x1="9" y1="9" x2="15" y2="15"/>
                            </svg>
                            {err}
                        </div>
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}

            // Hero Section
            <div class="task-detail-hero">
                <div class="task-detail-hero-left">
                    {move || {
                        if is_editing.get() {
                            view! {
                                <input
                                    type="text"
                                    class="task-detail-title-input"
                                    prop:value=move || edit_data.get().task_name.clone()
                                    on:input=move |ev| update_edit_field("task_name", event_target_value(&ev))
                                    placeholder="Task name..."
                                />
                            }.into_any()
                        } else {
                            view! {
                                <h1 class="task-detail-title">{move || current_task().task_name.clone()}</h1>
                            }.into_any()
                        }
                    }}
                    <div class="task-detail-badges">
                        {move || {
                            let t = current_task();
                            let sc = status_class(&t.task_status);
                            let sl = status_label(&t.task_status);
                            let pc = priority_class(t.task_priority);
                            let pl = priority_label(t.task_priority);
                            view! {
                                <span class=format!("task-detail-badge status-badge {}", sc)>
                                    {match t.task_status {
                                        TaskStatus::Active => view! {
                                            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                                                <circle cx="12" cy="12" r="10"/>
                                                <polyline points="12 6 12 12 16 14"/>
                                            </svg>
                                        }.into_any(),
                                        TaskStatus::Completed => view! {
                                            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                                                <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
                                                <polyline points="22 4 12 14.01 9 11.01"/>
                                            </svg>
                                        }.into_any(),
                                        TaskStatus::Paused => view! {
                                            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                                                <rect x="6" y="4" width="4" height="16"/>
                                                <rect x="14" y="4" width="4" height="16"/>
                                            </svg>
                                        }.into_any(),
                                    }}
                                    {sl}
                                </span>
                                <span class=format!("task-detail-badge priority-badge {}", pc)>{pl}</span>
                                {if t.is_favorite {
                                    view! {
                                        <span class="task-detail-badge favorite-badge favorited">
                                            <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="1">
                                                <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
                                            </svg>
                                            "Favorite"
                                        </span>
                                    }.into_any()
                                } else {
                                    ().into_any()
                                }}
                            }
                        }}
                    </div>
                </div>
                <div class="task-detail-hero-right">
                    <div class="task-detail-progress-ring" style=move || {
                        let pct = status_progress(&current_task().task_status);
                        format!("--progress: {}%", pct)
                    }>
                        <svg viewBox="0 0 48 48">
                            <circle class="progress-ring-bg" cx="24" cy="24" r="20" fill="none" stroke-width="4"/>
                            <circle class="progress-ring-fill" cx="24" cy="24" r="20" fill="none" stroke-width="4"
                                style=move || format!("stroke-dashoffset: {};", 125.66 * (1.0 - status_progress(&current_task().task_status) as f64 / 100.0))
                            />
                        </svg>
                        <span class="progress-ring-text">{move || format!("{}%", status_progress(&current_task().task_status))}</span>
                    </div>
                </div>
            </div>

            // Progress Bar
            <div class="task-detail-progress-bar">
                <div class="progress-bar-track">
                    {move || {
                        let pct = status_progress(&current_task().task_status);
                        view! {
                            <div
                                class="progress-bar-fill"
                                style=format!("width: {}%;", pct)
                            />
                        }
                    }}
                </div>
            </div>

            // Two-Column Layout
            <div class="task-detail-layout">
                // Left Column - Main Content
                <div class="task-detail-main">
                    // Description Card
                    <div class="task-detail-card">
                        <div class="task-detail-card-header">
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <line x1="17" y1="10" x2="3" y2="10"/>
                                <line x1="21" y1="6" x2="3" y2="6"/>
                                <line x1="21" y1="14" x2="3" y2="14"/>
                                <line x1="17" y1="18" x2="3" y2="18"/>
                            </svg>
                            <h3 class="task-detail-card-title">"Description"</h3>
                        </div>
                        <div class="task-detail-card-body">
                            {move || {
                                if is_editing.get() {
                                    view! {
                                        <textarea
                                            class="task-detail-textarea"
                                            prop:value=move || edit_data.get().task_description.clone().unwrap_or_default()
                                            on:input=move |ev| update_edit_field("task_description", event_target_value(&ev))
                                            placeholder="Add a description..."
                                            rows="4"
                                        />
                                    }.into_any()
                                } else {
                                    let desc = current_task().task_description.clone().unwrap_or_default();
                                    if desc.is_empty() {
                                        view! {
                                            <p class="task-detail-empty-desc">"No description provided."</p>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <p class="task-detail-desc">{desc}</p>
                                        }.into_any()
                                    }
                                }
                            }}
                        </div>
                    </div>

                    // Tags Card
                    <div class="task-detail-card">
                        <div class="task-detail-card-header">
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/>
                                <line x1="7" y1="7" x2="7.01" y2="7"/>
                            </svg>
                            <h3 class="task-detail-card-title">"Tags"</h3>
                        </div>
                        <div class="task-detail-card-body">
                            {move || {
                                let kws: Vec<String> = current_task().task_keywords.iter().cloned().collect();
                                if kws.is_empty() {
                                    view! {
                                        <p class="task-detail-empty-desc">"No tags assigned."</p>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="tag-chips">
                                            {kws.into_iter().map(|k| view! {
                                                <span class="tag-chip">{k}</span>
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>
                    </div>

                    // History Card
                    <div class="task-detail-card">
                        <div class="task-detail-card-header">
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <polyline points="1 4 1 10 7 10"/>
                                <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"/>
                            </svg>
                            <h3 class="task-detail-card-title">"Activity History"</h3>
                            <span class="task-detail-card-count">
                                {move || task_logs.get().len()}
                            </span>
                        </div>
                        <div class="task-detail-card-body">
                            <div class="task-history-list">
                                {move || {
                                    let logs = task_logs.get();
                                    if logs.is_empty() {
                                        view! {
                                            <div class="task-detail-empty-state">
                                                <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                                                    <polyline points="1 4 1 10 7 10"/>
                                                    <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"/>
                                                </svg>
                                                <p>"No activity recorded yet."</p>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            {logs.into_iter().map(|log| {
                                                let timestamp = log.get("created_at").and_then(|v| v.as_i64()).unwrap_or(0);
                                                let action = log.get("action").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
                                                let details = log.get("details").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                                let operator_id = log.get("operator_id").and_then(|v| v.as_u64()).unwrap_or(0);
                                                let operator_name = operator_names.get().get(&operator_id).cloned().unwrap_or_else(|| format!("User {}", operator_id));
                                                
                                                view! {
                                                    <div class="task-history-item">
                                                        <div class="task-history-dot"></div>
                                                        <div class="task-history-body">
                                                            <div class="task-history-top">
                                                                <span class="task-history-action">
                                                                    {format_action_type(&action)}
                                                                </span>
                                                                <span class="task-history-time">
                                                                    {format_log_timestamp(timestamp)}
                                                                </span>
                                                            </div>
                                                            <span class="task-history-details">
                                                                {"by "}{operator_name}
                                                                {if details.is_empty() { "".to_string() } else { format!(" — {}", details) }}
                                                            </span>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        }.into_any()
                                    }
                                }}
                            </div>
                        </div>
                    </div>

                    // Comments Card
                    <TaskComments task_id=task_id />
                </div>

                // Right Column - Sidebar
                <div class="task-detail-sidebar">
                    // Status Card
                    <div class="task-detail-card task-detail-sidebar-card">
                        <div class="task-detail-card-header">
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <circle cx="12" cy="12" r="10"/>
                                <polyline points="12 6 12 12 16 14"/>
                            </svg>
                            <h3 class="task-detail-card-title">"Status"</h3>
                        </div>
                        <div class="task-detail-card-body">
                            {move || {
                                if is_editing.get() {
                                    view! {
                                        <div class="task-detail-status-switch">
                                            <button
                                                class=format!("status-option {}", if edit_data.get().task_status == TaskStatus::Active { "active" } else { "" })
                                                on:click=move |_| edit_data.update(|d| d.task_status = TaskStatus::Active)
                                            >
                                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                                                "Active"
                                            </button>
                                            <button
                                                class=format!("status-option {}", if edit_data.get().task_status == TaskStatus::Paused { "active" } else { "" })
                                                on:click=move |_| edit_data.update(|d| d.task_status = TaskStatus::Paused)
                                            >
                                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></svg>
                                                "Paused"
                                            </button>
                                            <button
                                                class=format!("status-option completed {}", if edit_data.get().task_status == TaskStatus::Completed { "active" } else { "" })
                                                on:click=move |_| edit_data.update(|d| d.task_status = TaskStatus::Completed)
                                            >
                                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
                                                "Done"
                                            </button>
                                        </div>
                                    }.into_any()
                                } else {
                                    let t = current_task();
                                    let sc = status_class(&t.task_status);
                                    let sl = status_label(&t.task_status);
                                    view! {
                                        <div class=format!("task-detail-status-display {}", sc)>
                                            {match t.task_status {
                                                TaskStatus::Active => view! {
                                                    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                                                }.into_any(),
                                                TaskStatus::Completed => view! {
                                                    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
                                                }.into_any(),
                                                TaskStatus::Paused => view! {
                                                    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></svg>
                                                }.into_any(),
                                            }}
                                            <span>{sl}</span>
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>
                    </div>

                    // Details Card
                    <div class="task-detail-card task-detail-sidebar-card">
                        <div class="task-detail-card-header">
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                                <polyline points="14 2 14 8 20 8"/>
                                <line x1="16" y1="13" x2="8" y2="13"/>
                                <line x1="16" y1="17" x2="8" y2="17"/>
                                <polyline points="10 9 9 9 8 9"/>
                            </svg>
                            <h3 class="task-detail-card-title">"Details"</h3>
                        </div>
                        <div class="task-detail-card-body">
                            <div class="task-detail-fields">
                                // Priority
                                <div class="task-detail-field">
                                    <div class="task-detail-field-icon priority-icon">
                                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                            <path d="M12 2L2 7l10 5 10-5-10-5z"/>
                                            <path d="M2 17l10 5 10-5"/>
                                            <path d="M2 12l10 5 10-5"/>
                                        </svg>
                                    </div>
                                    <div class="task-detail-field-content">
                                        <span class="task-detail-field-label">"Priority"</span>
                                        {move || {
                                            if is_editing.get() {
                                                view! {
                                                    <div class="task-detail-priority-selector">
                                                        {(0..=10_u8).step_by(1).map(|v| {
                                                            let label = priority_label(v);
                                                            let cls = priority_class(v);
                                                            let is_selected = move || edit_data.get().task_priority == v;
                                                            view! {
                                                                <button
                                                                    class=format!("priority-pip {} {}", cls, if is_selected() { "selected" } else { "" })
                                                                    on:click=move |_| edit_data.update(|d| d.task_priority = v)
                                                                    title=label
                                                                >
                                                                    {v}
                                                                </button>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                }.into_any()
                                            } else {
                                                let t = current_task();
                                                let pc = priority_class(t.task_priority);
                                                let pl = priority_label(t.task_priority);
                                                view! {
                                                    <span class=format!("task-detail-field-value {}", pc)>{pl}</span>
                                                }.into_any()
                                            }
                                        }}
                                    </div>
                                </div>

                                // Difficulty
                                <div class="task-detail-field">
                                    <div class="task-detail-field-icon difficulty-icon">
                                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                            <line x1="12" y1="20" x2="12" y2="10"/>
                                            <line x1="18" y1="20" x2="18" y2="4"/>
                                            <line x1="6" y1="20" x2="6" y2="16"/>
                                        </svg>
                                    </div>
                                    <div class="task-detail-field-content">
                                        <span class="task-detail-field-label">"Difficulty"</span>
                                        {move || {
                                            if is_editing.get() {
                                                view! {
                                                    <input
                                                        type="number"
                                                        class="task-detail-inline-input"
                                                        min="0"
                                                        max="10"
                                                        prop:value=move || edit_data.get().task_difficulty.to_string()
                                                        on:input=move |ev| update_edit_field("task_difficulty", event_target_value(&ev))
                                                    />
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <span class="task-detail-field-value">
                                                        {move || {
                                                            let d = current_task().task_difficulty;
                                                            format!("{}/10", d)
                                                        }}
                                                    </span>
                                                }.into_any()
                                            }
                                        }}
                                    </div>
                                </div>

                                // Deadline
                                <div class="task-detail-field">
                                    <div class="task-detail-field-icon deadline-icon">
                                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                            <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/>
                                            <line x1="16" y1="2" x2="16" y2="6"/>
                                            <line x1="8" y1="2" x2="8" y2="6"/>
                                            <line x1="3" y1="10" x2="21" y2="10"/>
                                        </svg>
                                    </div>
                                    <div class="task-detail-field-content">
                                        <span class="task-detail-field-label">"Deadline"</span>
                                        {move || {
                                            if is_editing.get() {
                                                view! {
                                                    <input
                                                        type="date"
                                                        class="task-detail-inline-input"
                                                        prop:value=move || {
                                                            edit_data.get().task_deadline.map(|ts| {
                                                                let dt = chrono::DateTime::from_timestamp(ts, 0)
                                                                    .unwrap_or_default()
                                                                    .date_naive();
                                                                dt.format("%Y-%m-%d").to_string()
                                                            }).unwrap_or_default()
                                                        }
                                                        on:input=move |ev| update_edit_field("task_deadline", event_target_value(&ev))
                                                    />
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <span class="task-detail-field-value">
                                                        {move || {
                                                            current_task().task_deadline.map(|ts| format_timestamp(ts)).unwrap_or_else(|| "Not set".to_string())
                                                        }}
                                                    </span>
                                                }.into_any()
                                            }
                                        }}
                                    </div>
                                </div>

                                // Team
                                {move || {
                                    let t = current_task();
                                    if let Some(_team_id) = t.task_team_id {
                                        view! {
                                            <div class="task-detail-field">
                                                <div class="task-detail-field-icon team-icon">
                                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                                        <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
                                                        <circle cx="9" cy="7" r="4"/>
                                                        <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
                                                        <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
                                                    </svg>
                                                </div>
                                                <div class="task-detail-field-content">
                                                    <span class="task-detail-field-label">"Team"</span>
                                                    <span class="task-detail-field-value">{format!("Team #{}", _team_id)}</span>
                                                </div>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="task-detail-field">
                                                <div class="task-detail-field-icon team-icon">
                                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                                        <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                                                        <circle cx="12" cy="7" r="4"/>
                                                    </svg>
                                                </div>
                                                <div class="task-detail-field-content">
                                                    <span class="task-detail-field-label">"Team"</span>
                                                    <span class="task-detail-field-value dim">"Personal task"</span>
                                                </div>
                                            </div>
                                        }.into_any()
                                    }
                                }}
                            </div>
                        </div>
                    </div>

                    // Metadata Card
                    <div class="task-detail-card task-detail-sidebar-card">
                        <div class="task-detail-card-header">
                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <circle cx="12" cy="12" r="10"/>
                                <line x1="12" y1="16" x2="12" y2="12"/>
                                <line x1="12" y1="8" x2="12.01" y2="8"/>
                            </svg>
                            <h3 class="task-detail-card-title">"Information"</h3>
                        </div>
                        <div class="task-detail-card-body">
                            <div class="task-detail-info-list">
                                <div class="task-detail-info-item">
                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                        <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                                        <circle cx="12" cy="7" r="4"/>
                                    </svg>
                                    <div class="task-detail-info-content">
                                        <span class="task-detail-info-label">"Created By"</span>
                                        <span class="task-detail-info-value">
                                            {move || creator.get().map(|u| u.username.clone()).unwrap_or_else(|| "Loading...".to_string())}
                                        </span>
                                    </div>
                                </div>
                                <div class="task-detail-info-item">
                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                        <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/>
                                        <line x1="16" y1="2" x2="16" y2="6"/>
                                        <line x1="8" y1="2" x2="8" y2="6"/>
                                        <line x1="3" y1="10" x2="21" y2="10"/>
                                    </svg>
                                    <div class="task-detail-info-content">
                                        <span class="task-detail-info-label">"Created At"</span>
                                        <span class="task-detail-info-value">{move || format_timestamp(current_task().task_create_time)}</span>
                                    </div>
                                </div>
                                <div class="task-detail-info-item">
                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                        <polyline points="1 4 1 10 7 10"/>
                                        <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"/>
                                    </svg>
                                    <div class="task-detail-info-content">
                                        <span class="task-detail-info-label">"Updated At"</span>
                                        <span class="task-detail-info-value">
                                            {move || current_task().task_update_time.map(|ts| format_timestamp(ts)).unwrap_or_else(|| "N/A".to_string())}
                                        </span>
                                    </div>
                                </div>
                                <div class="task-detail-info-item">
                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                        <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
                                        <polyline points="22 4 12 14.01 9 11.01"/>
                                    </svg>
                                    <div class="task-detail-info-content">
                                        <span class="task-detail-info-label">"Completed At"</span>
                                        <span class="task-detail-info-value">
                                            {move || current_task().task_complete_time.map(|ts| format_timestamp(ts)).unwrap_or_else(|| "N/A".to_string())}
                                        </span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <Modal
                title="Confirm Delete".to_string()
                open=MaybeSignal::derive(move || show_delete_confirm.get())
                close_on_overlay=false
                on_close=cancel_delete
            >
                <div class="delete-confirm-content">
                    <p class="delete-confirm-message">"Are you sure you want to delete this task? This action cannot be undone."</p>
                    <div class="delete-confirm-actions">
                        <Button
                            variant=ButtonVariant::Secondary
                            size=ButtonSize::Sm
                            on_click=cancel_delete
                        >
                            "Cancel"
                        </Button>
                        <Button
                            variant=ButtonVariant::Danger
                            size=ButtonSize::Sm
                            on_click=confirm_delete
                        >
                            "Delete"
                        </Button>
                    </div>
                </div>
            </Modal>
        </div>
    }
}
