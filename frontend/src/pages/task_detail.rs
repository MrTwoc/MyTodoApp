use crate::api::task::{delete_task as api_delete_task, get_task as api_get_task, get_task_logs as api_get_task_logs, update_task as api_update_task};
use crate::api::user::get_user as api_get_user;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::card::Card;
use crate::components::modal::Modal;
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

    let load_task = {
        let client = client.clone();
        let task_id = task_id;
        let set_task = set_task.clone();
        let set_creator = set_creator.clone();
        let set_task_logs = set_task_logs.clone();
        let edit_data = edit_data.clone();
        let is_loading = is_loading.clone();
        move || {
            let client = client.clone();
            let task_id = task_id;
            let set_task = set_task.clone();
            let set_creator = set_creator.clone();
            let set_task_logs = set_task_logs.clone();
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
                        set_task_logs.set(logs);
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
        <div class="page">
            <header class="page-header task-detail-header">
                <div>
                    <button class="back-btn" on:click=nav_back>"← Back"</button>
                    {move || {
                        if is_editing.get() {
                            view! {
                                <input
                                    type="text"
                                    class="input-field task-title-input"
                                    prop:value=move || edit_data.get().task_name.clone()
                                    on:input=move |ev| update_edit_field("task_name", event_target_value(&ev))
                                />
                            }.into_any()
                        } else {
                            view! {
                                <h1 class="page-title task-detail-title">{move || current_task().task_name.clone()}</h1>
                            }.into_any()
                        }
                    }}
                </div>
                <div class="task-detail-actions">
                    {move || {
                        if is_editing.get() {
                            let saving = is_saving.get();
                            view! {
                                <Button
                                    variant=ButtonVariant::Primary
                                    size=ButtonSize::Sm
                                    disabled=saving
                                    on_click=save_edit
                                >
                                    {if saving { "Saving..." } else { "Save" }}
                                </Button>
                                <Button
                                    variant=ButtonVariant::Secondary
                                    size=ButtonSize::Sm
                                    disabled=saving
                                    on_click=cancel_edit
                                >
                                    "Cancel"
                                </Button>
                            }
                        } else {
                            view! {
                                <Button
                                    variant=ButtonVariant::Secondary
                                    size=ButtonSize::Sm
                                    on_click=start_edit
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
                            }
                        }
                    }}
                </div>
            </header>

            {move || {
                if let Some(err) = save_error.get() {
                    view! {
                        <div class="error-toast">{err}</div>
                    }.into_any()
                } else {
                    ().into_any()
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
                        <span class=format!("task-status-badge {}", sc)>{sl}</span>
                        <span class=format!("task-priority-badge {}", pc)>{pl}</span>
                    }
                }}
            </div>

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

            <Card title="Status".to_string()>
                <div class="task-detail-section">
                    {move || {
                        if is_editing.get() {
                            view! {
                                <select
                                    class="input-field"
                                    prop:value=move || status_label(&edit_data.get().task_status).to_string()
                                    on:change=move |ev| {
                                        let value = event_target_value(&ev);
                                        edit_data.update(|data| {
                                            data.task_status = match value.as_str() {
                                                "Active" => TaskStatus::Active,
                                                "Completed" => TaskStatus::Completed,
                                                "Paused" => TaskStatus::Paused,
                                                _ => data.task_status.clone(),
                                            };
                                        });
                                    }
                                >
                                    <option value="Active">Active</option>
                                    <option value="Completed">Completed</option>
                                    <option value="Paused">Paused</option>
                                </select>
                            }.into_any()
                        } else {
                            view! {
                                <p class="task-detail-value">{move || status_label(&current_task().task_status)}</p>
                            }.into_any()
                        }
                    }}
                </div>
            </Card>

            <Card title="Metadata".to_string()>
                <div class="task-detail-meta">
                    <div class="task-detail-meta-item">
                        <span class="meta-label">"Created By"</span>
                        <span class="meta-value">
                            {move || creator.get().map(|u| u.username.clone()).unwrap_or_else(|| "Loading...".to_string())}
                        </span>
                    </div>
                    <div class="task-detail-meta-item">
                        <span class="meta-label">"Created At"</span>
                        <span class="meta-value">{move || format_timestamp(current_task().task_create_time)}</span>
                    </div>
                    <div class="task-detail-meta-item">
                        <span class="meta-label">"Updated At"</span>
                        <span class="meta-value">
                            {move || current_task().task_update_time.map(|ts| format_timestamp(ts)).unwrap_or_else(|| "N/A".to_string())}
                        </span>
                    </div>
                    <div class="task-detail-meta-item">
                        <span class="meta-label">"Completed At"</span>
                        <span class="meta-value">
                            {move || current_task().task_complete_time.map(|ts| format_timestamp(ts)).unwrap_or_else(|| "N/A".to_string())}
                        </span>
                    </div>
                    <div class="task-detail-meta-item">
                        <span class="meta-label">"Favorite"</span>
                        <span class="meta-value">{move || if current_task().is_favorite { "★ Yes" } else { "☆ No" }}</span>
                    </div>
                </div>
            </Card>

            <Card title="Details".to_string()>
                <div class="task-detail-section">
                    <h4 class="task-detail-label">"Description"</h4>
                    {move || {
                        if is_editing.get() {
                            view! {
                                <textarea
                                    class="input-field task-description-input"
                                    prop:value=move || edit_data.get().task_description.clone().unwrap_or_default()
                                    on:input=move |ev| update_edit_field("task_description", event_target_value(&ev))
                                />
                            }.into_any()
                        } else {
                            view! {
                                <p class="task-detail-value">{move || current_task().task_description.clone().unwrap_or_default()}</p>
                            }.into_any()
                        }
                    }}
                </div>

                <div class="task-detail-section">
                    <h4 class="task-detail-label">"Deadline"</h4>
                    {move || {
                        if is_editing.get() {
                            view! {
                                <input
                                    type="date"
                                    class="input-field date-input"
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
                                <p class="task-detail-value">{move || {
                                    current_task().task_deadline.map(|ts| format_timestamp(ts)).unwrap_or_default()
                                }}</p>
                            }.into_any()
                        }
                    }}
                </div>

                <div class="task-detail-section">
                    <h4 class="task-detail-label">"Priority"</h4>
                    {move || {
                        if is_editing.get() {
                            view! {
                                <input
                                    type="number"
                                    class="input-field"
                                    min="0"
                                    max="10"
                                    prop:value=move || edit_data.get().task_priority.to_string()
                                    on:input=move |ev| update_edit_field("task_priority", event_target_value(&ev))
                                />
                            }.into_any()
                        } else {
                            view! {
                                <p class="task-detail-value">{move || current_task().task_priority.to_string()}</p>
                            }.into_any()
                        }
                    }}
                </div>

                <div class="task-detail-section">
                    <h4 class="task-detail-label">"Difficulty"</h4>
                    {move || {
                        if is_editing.get() {
                            view! {
                                <input
                                    type="number"
                                    class="input-field"
                                    min="0"
                                    max="10"
                                    prop:value=move || edit_data.get().task_difficulty.to_string()
                                    on:input=move |ev| update_edit_field("task_difficulty", event_target_value(&ev))
                                />
                            }.into_any()
                        } else {
                            view! {
                                <p class="task-detail-value">{move || current_task().task_difficulty.to_string()}</p>
                            }.into_any()
                        }
                    }}
                </div>

                <div class="task-detail-section">
                    <h4 class="task-detail-label">"Tags"</h4>
                    {move || {
                        let kws: Vec<String> = current_task().task_keywords.iter().cloned().collect();
                        if kws.is_empty() {
                            view! {
                                <p class="task-detail-value">"No tags"</p>
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
            </Card>

            <Card title="History".to_string()>
                <div class="task-history-list">
                    {move || {
                        let logs = task_logs.get();
                        if logs.is_empty() {
                            view! {
                                <p class="task-detail-empty">"No history available."</p>
                            }.into_any()
                        } else {
                            view! {
                                {logs.into_iter().map(|log| {
                                    let timestamp = log.get("created_at").and_then(|v| v.as_i64()).unwrap_or(0);
                                    let action = log.get("action").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
                                    let details = log.get("details").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    let operator_id = log.get("operator_id").and_then(|v| v.as_u64()).unwrap_or(0);
                                    
                                    view! {
                                        <div class="task-history-item">
                                            <div class="task-history-time">
                                                {format_log_timestamp(timestamp)}
                                            </div>
                                            <div class="task-history-content">
                                                <span class="task-history-action">
                                                    {format_action_type(&action)}
                                                </span>
                                                <span class="task-history-details">
                                                    {details}
                                                </span>
                                                <span class="task-history-operator">
                                                    {"Operator: "}{operator_id}
                                                </span>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            }.into_any()
                        }
                    }}
                </div>
            </Card>

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
