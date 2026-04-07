use crate::components::modal::Modal;
use crate::store::task_store::Task;
use leptos::ev;
use leptos::prelude::*;

#[derive(Clone, PartialEq)]
pub enum TaskFormMode {
    Create,
    Edit,
}

#[derive(Clone)]
pub struct TaskFormData {
    pub task_id: Option<u64>,
    pub task_name: String,
    pub task_description: Option<String>,
    pub task_keywords: Vec<String>,
    pub task_priority: u8,
    pub task_difficulty: u8,
    pub task_deadline: Option<i64>,
    pub task_leader_id: u64,
    pub task_team_id: Option<u64>,
}

impl Default for TaskFormData {
    fn default() -> Self {
        Self {
            task_id: None,
            task_name: String::new(),
            task_description: None,
            task_keywords: Vec::new(),
            task_priority: 5,
            task_difficulty: 0,
            task_deadline: None,
            task_leader_id: 0,
            task_team_id: None,
        }
    }
}

impl From<Task> for TaskFormData {
    fn from(task: Task) -> Self {
        Self {
            task_id: Some(task.task_id),
            task_name: task.task_name,
            task_description: task.task_description,
            task_keywords: task.task_keywords.into_iter().collect(),
            task_priority: task.task_priority,
            task_difficulty: task.task_difficulty,
            task_deadline: task.task_deadline,
            task_leader_id: task.task_leader_id,
            task_team_id: task.task_team_id,
        }
    }
}

#[component]
pub fn TaskFormModal(
    #[prop(default = MaybeSignal::Static(false))] open: MaybeSignal<bool>,
    #[prop(default = TaskFormMode::Create)] mode: TaskFormMode,
    #[prop(default = TaskFormData::default())] initial_data: TaskFormData,
    #[prop(default = false)] force_team_task: bool,
    #[prop(default = false)] offline_mode: bool,
    #[prop(default = None)] active_team_id: Option<u64>,
    #[prop(optional)] on_submit: Option<Callback<(TaskFormData,)>>,
    #[prop(optional)] on_close: Option<Callback<()>>,
) -> impl IntoView {
    let title = if mode == TaskFormMode::Create {
        "Create Task"
    } else {
        "Edit Task"
    };
    let submit_text = if mode == TaskFormMode::Create {
        "Create"
    } else {
        "Save"
    };

    let is_team_task = RwSignal::new(initial_data.task_team_id.is_some() || force_team_task);

    Effect::new(move |_| {
        if !force_team_task {
            is_team_task.set(initial_data.task_team_id.is_some());
        }
    });

    // let can_select_team = !offline_mode;
    let can_select_team = true;

    let initial_data_clone = initial_data.clone();
    let form_data = RwSignal::new(initial_data_clone);

    // 当 initial_data 属性变化时更新表单数据（简单起见，仅首次渲染后更新）
    Effect::new(move |_| {
        form_data.set(initial_data.clone());
    });

    // 字段更新函数
    let update_name = move |ev: ev::Event| {
        let value = event_target_value(&ev);
        form_data.update(|data| data.task_name = value);
    };

    let update_description = move |ev: ev::Event| {
        let value = event_target_value(&ev);
        form_data.update(|data| {
            data.task_description = if value.is_empty() { None } else { Some(value) }
        });
    };

    let update_priority = move |ev: ev::Event| {
        let value = event_target_value(&ev);
        if let Ok(priority) = value.parse::<u8>() {
            form_data.update(|data| data.task_priority = priority.min(10));
        }
    };

    let update_difficulty = move |ev: ev::Event| {
        let value = event_target_value(&ev);
        if let Ok(difficulty) = value.parse::<u8>() {
            form_data.update(|data| data.task_difficulty = difficulty.min(10));
        }
    };

    // 关键词输入：逗号分隔的字符串
    let update_keywords = move |ev: ev::Event| {
        let value = event_target_value(&ev);
        let keywords: Vec<String> = value
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        form_data.update(|data| data.task_keywords = keywords);
    };

    // 截止日期：将日期字符串转换为 i64 时间戳（UTC 午夜）
    let update_deadline = move |ev: ev::Event| {
        let value = event_target_value(&ev);
        if value.is_empty() {
            form_data.update(|data| data.task_deadline = None);
        } else {
            // 将 YYYY-MM-DD 转换为 Unix 时间戳（毫秒）
            if let Ok(date) = chrono::NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
                let timestamp = date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
                form_data.update(|data| data.task_deadline = Some(timestamp));
            }
        }
    };

    // 负责人 ID 和团队 ID 暂不提供前端输入（由后端决定）
    // 提交处理
    let handle_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        let mut data = form_data.get_untracked();
        if !is_team_task.get_untracked() {
            data.task_team_id = None;
        }
        if let Some(callback) = on_submit {
            callback.run((data,));
        }
    };

    // 取消处理
    let handle_cancel = move |_| {
        if let Some(callback) = on_close {
            callback.run(());
        }
    };

    view! {
        <Modal open=open title={title.to_string()}>
            <form class="form" on:submit=handle_submit>
                <div class="form-group">
                    <label class="form-label">Task Type</label>
                    <select
                        class="input-field"
                        prop:value=move || if is_team_task.get() { "team" } else { "personal" }
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            if value == "team" && can_select_team {
                                is_team_task.set(true);
                                if form_data.get_untracked().task_team_id.is_none() {
                                    form_data.update(|data| {
                                        data.task_team_id = active_team_id;
                                    });
                                }
                            } else {
                                is_team_task.set(false);
                            }
                        }
                    >
                        <option value="personal">Personal</option>
                        <option value="team" disabled=!can_select_team>
                            {if can_select_team { "Team" } else { "Team (offline unavailable)" }}
                        </option>
                    </select>
                </div>

                <div class="form-group">
                    <label class="form-label">Task Name</label>
                    <input
                        type="text"
                        class="input-field"
                        placeholder="Enter task name"
                        prop:value=move || form_data.get().task_name
                        on:input=update_name
                    />
                </div>

                <div class="form-group">
                    <label class="form-label">Description</label>
                    <textarea
                        class="input-field task-description-input"
                        placeholder="Enter task description"
                        prop:value=move || form_data.get().task_description.clone().unwrap_or_default()
                        on:input=update_description
                    />
                </div>

                <div class="form-group">
                    <label class="form-label">Keywords (comma separated)</label>
                    <input
                        type="text"
                        class="input-field"
                        placeholder="e.g., urgent, bug, feature"
                        prop:value=move || form_data.get().task_keywords.join(", ")
                        on:input=update_keywords
                    />
                </div>

                <div class="form-group">
                    <label class="form-label">Priority (0-10)</label>
                    <input
                        type="number"
                        class="input-field"
                        min="0"
                        max="10"
                        prop:value=move || form_data.get().task_priority.to_string()
                        on:input=update_priority
                    />
                </div>

                <div class="form-group">
                    <label class="form-label">Difficulty (0-10)</label>
                    <input
                        type="number"
                        class="input-field"
                        min="0"
                        max="10"
                        prop:value=move || form_data.get().task_difficulty.to_string()
                        on:input=update_difficulty
                    />
                </div>

                <div class="form-group">
                    <label class="form-label">Deadline</label>
                    <input
                        type="date"
                        class="input-field date-input"
                        prop:value=move || {
                            form_data.get().task_deadline.map(|ts| {
                                // 将毫秒时间戳转换为 YYYY‑MM‑DD 格式
                                let dt = chrono::DateTime::from_timestamp(ts,0)
                                    .unwrap_or_default()
                                    .date_naive();
                                dt.format("%Y-%m-%d").to_string()
                            }).unwrap_or_default()
                        }
                        on:input=update_deadline
                    />
                </div>

                <div class="form-actions">
                    <button type="button" class="btn btn-secondary btn-md" on:click=handle_cancel>
                        Cancel
                    </button>
                    <button type="submit" class="btn btn-primary btn-md">
                        {submit_text}
                    </button>
                </div>
            </form>
        </Modal>
    }
}

#[component]
pub fn TaskForm(
    #[prop(default = TaskFormMode::Create)] mode: TaskFormMode,
    #[prop(default = TaskFormData::default())] initial_data: TaskFormData,
    #[prop(default = false)] force_team_task: bool,
    #[prop(default = false)] offline_mode: bool,
    #[prop(default = None)] active_team_id: Option<u64>,
    #[prop(optional)] on_submit: Option<Callback<(TaskFormData,)>>,
    #[prop(optional)] on_cancel: Option<Callback<()>>,
) -> impl IntoView {
    let submit_text = if mode == TaskFormMode::Create {
        "Create"
    } else {
        "Save"
    };

    let is_team_task = RwSignal::new(initial_data.task_team_id.is_some() || force_team_task);

    Effect::new(move |_| {
        if !force_team_task {
            is_team_task.set(initial_data.task_team_id.is_some());
        }
    });

    // let can_select_team = !offline_mode;
    let can_select_team = true;

    let initial_data_clone = initial_data.clone();
    let form_data = RwSignal::new(initial_data_clone);

    // 当 initial_data 属性变化时更新表单数据
    Effect::new(move |_| {
        form_data.set(initial_data.clone());
    });

    // 字段更新函数（与 TaskFormModal 相同）
    let update_name = move |ev: ev::Event| {
        let value = event_target_value(&ev);
        form_data.update(|data| data.task_name = value);
    };

    let update_description = move |ev: ev::Event| {
        let value = event_target_value(&ev);
        form_data.update(|data| {
            data.task_description = if value.is_empty() { None } else { Some(value) }
        });
    };

    let update_priority = move |ev: ev::Event| {
        let value = event_target_value(&ev);
        if let Ok(priority) = value.parse::<u8>() {
            form_data.update(|data| data.task_priority = priority.min(10));
        }
    };

    let update_difficulty = move |ev: ev::Event| {
        let value = event_target_value(&ev);
        if let Ok(difficulty) = value.parse::<u8>() {
            form_data.update(|data| data.task_difficulty = difficulty.min(10));
        }
    };

    let update_keywords = move |ev: ev::Event| {
        let value = event_target_value(&ev);
        let keywords: Vec<String> = value
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        form_data.update(|data| data.task_keywords = keywords);
    };

    let update_deadline = move |ev: ev::Event| {
        let value = event_target_value(&ev);
        if value.is_empty() {
            form_data.update(|data| data.task_deadline = None);
        } else {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
                let timestamp = date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
                form_data.update(|data| data.task_deadline = Some(timestamp));
            }
        }
    };

    let handle_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        let mut data = form_data.get_untracked();
        if !is_team_task.get_untracked() {
            data.task_team_id = None;
        }
        if let Some(callback) = on_submit {
            callback.run((data,));
        }
    };

    let handle_cancel = move |_| {
        if let Some(callback) = on_cancel {
            callback.run(());
        }
    };

    view! {
        <form class="form" on:submit=handle_submit>
            <div class="form-group">
                <label class="form-label">Task Type</label>
                <select
                    class="input-field"
                    prop:value=move || if is_team_task.get() { "team" } else { "personal" }
                    on:change=move |ev| {
                            let value = event_target_value(&ev);
                            if value == "team" && can_select_team {
                                is_team_task.set(true);
                                if form_data.get_untracked().task_team_id.is_none() {
                                    form_data.update(|data| {
                                        data.task_team_id = active_team_id;
                                    });
                                }
                            } else {
                                is_team_task.set(false);
                            }
                        }
                >
                    <option value="personal">Personal</option>
                    <option value="team" disabled=!can_select_team>
                        {if can_select_team { "Team" } else { "Team (offline unavailable)" }}
                    </option>
                </select>
            </div>

            <div class="form-group">
                <label class="form-label">Task Name</label>
                <input
                    type="text"
                    class="input-field"
                    placeholder="Enter task name"
                    prop:value=move || form_data.get().task_name
                    on:input=update_name
                />
            </div>

            <div class="form-group">
                <label class="form-label">Description</label>
                <textarea
                    class="input-field task-description-input"
                    placeholder="Enter task description"
                    prop:value=move || form_data.get().task_description.clone().unwrap_or_default()
                    on:input=update_description
                />
            </div>

            <div class="form-group">
                <label class="form-label">Keywords (comma separated)</label>
                <input
                    type="text"
                    class="input-field"
                    placeholder="e.g., urgent, bug, feature"
                    prop:value=move || form_data.get().task_keywords.join(", ")
                    on:input=update_keywords
                />
            </div>

            <div class="form-group">
                <label class="form-label">Priority (0-10)</label>
                <input
                    type="number"
                    class="input-field"
                    min="0"
                    max="10"
                    prop:value=move || form_data.get().task_priority.to_string()
                    on:input=update_priority
                />
            </div>

            <div class="form-group">
                <label class="form-label">Difficulty (0-10)</label>
                <input
                    type="number"
                    class="input-field"
                    min="0"
                    max="10"
                    prop:value=move || form_data.get().task_difficulty.to_string()
                    on:input=update_difficulty
                />
            </div>

            <div class="form-group">
                <label class="form-label">Deadline</label>
                <input
                    type="date"
                    class="input-field date-input"
                    prop:value=move || {
                        form_data.get().task_deadline.map(|ts| {
                            let dt = chrono::DateTime::from_timestamp(ts,0)
                                .unwrap_or_default()
                                .date_naive();
                            dt.format("%Y-%m-%d").to_string()
                        }).unwrap_or_default()
                    }
                    on:input=update_deadline
                />
            </div>

            <div class="form-actions">
                <button type="button" class="btn btn-secondary btn-md" on:click=handle_cancel>
                    Cancel
                </button>
                <button type="submit" class="btn btn-primary btn-md">
                    {submit_text}
                </button>
            </div>
        </form>
    }
}
