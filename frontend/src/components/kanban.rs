use crate::store::task_store::{Task, TaskStatus};
use crate::store::use_team_store;
use leptos::ev;
use leptos::prelude::*;

#[component]
pub fn KanbanCard(
    task: Task,
    #[prop(default = false)] compact: bool,
    #[prop(optional)] on_click: Option<Callback<(ev::MouseEvent,)>>,
    #[prop(optional)] on_toggle_favorite: Option<Callback<(u64,)>>,
) -> impl IntoView {
    let priority_class = match task.task_priority {
        0..=2 => "priority-low",
        3..=5 => "priority-medium",
        6..=8 => "priority-high",
        _ => "priority-urgent",
    };

    let priority_label = match task.task_priority {
        0..=2 => "Low",
        3..=5 => "Medium",
        6..=8 => "High",
        _ => "Urgent",
    };

    let handle_click = move |ev: ev::MouseEvent| {
        if let Some(cb) = on_click.as_ref() {
            cb.run((ev,));
        }
    };

    let handle_favorite_click = move |ev: ev::MouseEvent| {
        ev.stop_propagation();
        if let Some(cb) = on_toggle_favorite.as_ref() {
            cb.run((task.task_id,));
        }
    };

    let team_store = use_team_store();
    let team_name = task.task_team_id.and_then(|tid| {
        team_store
            .state
            .get()
            .teams
            .iter()
            .find(|t| t.team_id == tid)
            .map(|t| t.team_name.clone())
    });

    let has_deadline = task.task_deadline.is_some();
    let has_description = task.task_description.is_some();
    let has_team = team_name.is_some();

    let deadline_text = task.task_deadline.map(|ts| {
        let date = js_sys::Date::new(&js_sys::Date::new_0());
        date.set_milliseconds(ts as u32);
        format!(
            "{}/{:02}/{:02}",
            date.get_full_year(),
            date.get_month() + 1,
            date.get_date()
        )
    });

    view! {
        <div
            class=format!("kanban-card {}", if compact { "compact" } else { "" })
            on:click=handle_click
        >
            <div class="kanban-card-header">
                <button
                    class=format!("favorite-btn {}", if task.is_favorite { "favorited" } else { "" })
                    title=if task.is_favorite { "Remove from favorites" } else { "Add to favorites" }
                    on:click=handle_favorite_click
                >
                    <svg viewBox="0 0 24 24" fill=if task.is_favorite { "currentColor" } else { "none" } stroke="currentColor" stroke-width="2">
                        <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                    </svg>
                </button>
            </div>
            <h3 class="kanban-card-title">{task.task_name}</h3>
            {if has_description {
                view! { <p class="kanban-card-desc">{task.task_description.as_deref().unwrap_or("")}</p> }.into_any()
            } else {
                ().into_any()
            }}
            <div class="kanban-card-meta">
                <span class=format!("priority-badge {}", priority_class)>
                    {priority_label}
                </span>
                {if has_deadline {
                    view! { <span class="deadline-badge">{deadline_text.unwrap_or_default()}</span> }.into_any()
                } else {
                    ().into_any()
                }}
                {if has_team {
                    view! { <span class="team-badge">{team_name.unwrap_or_default()}</span> }.into_any()
                } else {
                    ().into_any()
                }}
            </div>
        </div>
    }
}

pub struct KanbanColumnConfig {
    pub status: TaskStatus,
    pub title: &'static str,
    pub color: &'static str,
}

pub fn get_kanban_columns() -> Vec<KanbanColumnConfig> {
    vec![
        KanbanColumnConfig {
            status: TaskStatus::Active,
            title: "To Do",
            color: "#3b82f6",
        },
        KanbanColumnConfig {
            status: TaskStatus::Paused,
            title: "In Progress",
            color: "#f59e0b",
        },
        KanbanColumnConfig {
            status: TaskStatus::Completed,
            title: "Done",
            color: "#10b981",
        },
    ]
}

#[component]
pub fn KanbanColumn(
    config: KanbanColumnConfig,
    tasks: Vec<Task>,
    on_task_click: Callback<(u64,)>,
    on_task_toggle_favorite: Callback<(u64,)>,
) -> impl IntoView {
    let column_tasks = tasks;

    view! {
        <div class="kanban-column">
            <div class="kanban-column-header" style=format!("--column-color: {}", config.color)>
                <h2 class="kanban-column-title">{config.title}</h2>
                <span class="kanban-column-count">{column_tasks.len()}</span>
            </div>
            <div class="kanban-column-content">
                {column_tasks.into_iter().map(|task| {
                    let task_id = task.task_id;
                    let on_click = on_task_click.clone();
                    let on_fav = on_task_toggle_favorite.clone();
                    view! {
                        <KanbanCard
                            task=task
                            compact=true
                            on_click=Callback::from(move |_| on_click.run((task_id,)))
                            on_toggle_favorite=on_fav
                        />
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
pub fn KanbanBoard(
    tasks: Vec<Task>,
    on_task_click: Callback<(u64,)>,
    on_task_toggle_favorite: Callback<(u64,)>,
) -> impl IntoView {
    let columns = get_kanban_columns();

    let get_tasks_by_status = move |status: &TaskStatus| -> Vec<Task> {
        tasks
            .iter()
            .filter(|t| &t.task_status == status)
            .cloned()
            .collect()
    };

    view! {
        <div class="kanban-board">
            {columns.into_iter().map(|config| {
                let column_tasks = get_tasks_by_status(&config.status);
                view! {
                    <KanbanColumn
                        config=config
                        tasks=column_tasks
                        on_task_click=on_task_click.clone()
                        on_task_toggle_favorite=on_task_toggle_favorite.clone()
                    />
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
