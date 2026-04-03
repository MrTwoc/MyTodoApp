use std::collections::HashSet;

use crate::store::task_store::{Task, TaskFilters, TaskStatus};
use crate::store::{get_local_storage_item, set_local_storage_item};
use chrono::Utc;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

const OFFLINE_TASKS_KEY: &str = "todo_offline_tasks_v1";
const OFFLINE_MODE_KEY: &str = "todo_offline_mode_v1";

#[derive(Debug, Clone, PartialEq)]
pub struct OfflineTaskState {
    pub enabled: bool,
    pub tasks: Vec<Task>,
    pub error: Option<String>,
}

impl Default for OfflineTaskState {
    fn default() -> Self {
        Self {
            enabled: false,
            tasks: Vec::new(),
            error: None,
        }
    }
}

#[derive(Clone)]
pub struct OfflineTaskStore {
    pub state: ReadSignal<OfflineTaskState>,
    pub set_state: WriteSignal<OfflineTaskState>,
}

fn persisted_state() -> OfflineTaskState {
    let snapshot = get_local_storage_item(OFFLINE_TASKS_KEY)
        .and_then(|raw| serde_json::from_str::<Vec<Task>>(&raw).ok())
        .unwrap_or_default();

    let enabled = get_local_storage_item(OFFLINE_MODE_KEY)
        .is_some_and(|raw| raw.eq_ignore_ascii_case("true"));

    OfflineTaskState {
        enabled,
        tasks: snapshot,
        error: None,
    }
}

fn persist_state(tasks: &[Task], enabled: bool) {
    match serde_json::to_string(tasks) {
        Ok(payload) => set_local_storage_item(OFFLINE_TASKS_KEY, &payload),
        Err(_) => {
            set_local_storage_item(OFFLINE_TASKS_KEY, "[]");
        }
    }
    set_local_storage_item(OFFLINE_MODE_KEY, if enabled { "true" } else { "false" });
}

impl OfflineTaskStore {
    pub fn set_enabled(&self, enabled: bool) {
        let mut state = self.state.get();
        state.enabled = enabled;
        state.error = None;
        self.set_state.set(state.clone());
        persist_state(&state.tasks, enabled);
    }

    pub fn is_enabled(&self) -> bool {
        self.state.get().enabled
    }

    pub fn set_error(&self, error: String) {
        let mut state = self.state.get();
        state.error = Some(error);
        self.set_state.set(state);
    }

    pub fn add_task(&self, task: Task) {
        let mut state = self.state.get();
        state.tasks.push(task);
        state.error = None;
        self.set_state.set(state.clone());
        persist_state(&state.tasks, state.enabled);
    }

    pub fn update_task(&self, task_id: u64, updated: Task) {
        let mut state = self.state.get();
        if let Some(pos) = state.tasks.iter().position(|t| t.task_id == task_id) {
            state.tasks[pos] = updated;
            state.error = None;
            self.set_state.set(state.clone());
            persist_state(&state.tasks, state.enabled);
            return;
        }
        state.error = Some("Task not found".to_string());
        self.set_state.set(state);
    }

    pub fn delete_task(&self, task_id: u64) {
        let mut state = self.state.get();
        state.tasks.retain(|t| t.task_id != task_id);
        state.error = None;
        self.set_state.set(state.clone());
        persist_state(&state.tasks, state.enabled);
    }

    pub fn set_task_status(&self, task_id: u64, status: TaskStatus) {
        let mut state = self.state.get();
        let mut found = false;

        if let Some(task) = state.tasks.iter_mut().find(|t| t.task_id == task_id) {
            task.task_status = status;
            task.task_update_time = Some(Utc::now().timestamp());
            found = true;
        }

        if found {
            state.error = None;
            self.set_state.set(state.clone());
            persist_state(&state.tasks, state.enabled);
        } else {
            state.error = Some("Task not found".to_string());
            self.set_state.set(state);
        }
    }

    pub fn filtered_tasks(&self, filters: &TaskFilters) -> Vec<Task> {
        let tasks = self.state.get().tasks;
        let mut result = tasks;

        if let Some(status) = &filters.status {
            result.retain(|t| &t.task_status == status);
        }
        if let Some(min) = filters.priority_min {
            result.retain(|t| t.task_priority >= min);
        }
        if let Some(max) = filters.priority_max {
            result.retain(|t| t.task_priority <= max);
        }
        if let Some(team_id) = filters.team_id {
            result.retain(|t| t.task_team_id == Some(team_id));
        }
        if let Some(query) = &filters.search_query {
            let q = query.to_lowercase();
            result.retain(|t| {
                t.task_name.to_lowercase().contains(&q)
                    || t.task_description
                        .as_ref()
                        .is_some_and(|desc| desc.to_lowercase().contains(&q))
            });
        }

        result
    }

    pub fn new_task(
        &self,
        name: String,
        description: Option<String>,
        keywords: Vec<String>,
        priority: u8,
        deadline: Option<i64>,
    ) -> Task {
        let mut keyword_set = HashSet::new();
        for key in keywords {
            if !key.trim().is_empty() {
                keyword_set.insert(key.trim().to_string());
            }
        }
        Task {
            task_id: self.next_id(),
            task_name: name,
            task_description: description,
            task_keywords: keyword_set,
            task_priority: priority,
            task_deadline: deadline,
            task_complete_time: None,
            task_status: TaskStatus::Active,
            task_create_time: Utc::now().timestamp(),
            task_leader_id: 0,
            task_team_id: None,
            task_update_time: None,
        }
    }

    fn next_id(&self) -> u64 {
        let now = Utc::now().timestamp_millis() as u64;
        let count = self.state.get().tasks.len() as u64;
        now.saturating_mul(10_000)
            .saturating_add(count.saturating_add(1))
    }
}

pub fn create_offline_task_store() -> OfflineTaskStore {
    let state = persisted_state();
    let (state_signal, set_state) = signal(state);
    OfflineTaskStore {
        state: state_signal,
        set_state,
    }
}
