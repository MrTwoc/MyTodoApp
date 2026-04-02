use crate::components::modal::Modal;
use crate::store::task_store::Task;
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
            task_deadline: task.task_deadline,
            task_leader_id: task.task_leader_id,
            task_team_id: task.task_team_id,
        }
    }
}

#[component]
pub fn TaskFormModal(
    #[prop(default = false)] open: bool,
    #[prop(default = TaskFormMode::Create)] mode: TaskFormMode,
    #[prop(default = TaskFormData::default())] initial_data: TaskFormData,
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

    view! {
        <Modal open=MaybeSignal::Static(open) title={title.to_string()}>
            <form class="form">
                <div class="form-group">
                    <label class="form-label">Task Name</label>
                    <input type="text" class="input-field" placeholder="Enter task name" />
                </div>

                <div class="form-group">
                    <label class="form-label">Description</label>
                    <textarea class="input-field task-description-input" placeholder="Enter task description"></textarea>
                </div>

                <div class="form-group">
                    <label class="form-label">Priority</label>
                    <input type="number" class="input-field" min="1" max="10" />
                </div>

                <div class="form-group">
                    <label class="form-label">Deadline</label>
                    <input type="date" class="input-field date-input" />
                </div>

                <div class="form-actions">
                    <button type="button" class="btn btn-secondary btn-md">Cancel</button>
                    <button type="submit" class="btn btn-primary btn-md">{submit_text}</button>
                </div>
            </form>
        </Modal>
    }
}
