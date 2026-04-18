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

    let priority_indicator_class = match task.task_priority {
        0..=2 => "kanban-card-priority-indicator priority-indicator-low",
        3..=5 => "kanban-card-priority-indicator priority-indicator-medium",
        6..=8 => "kanban-card-priority-indicator priority-indicator-high",
        _ => "kanban-card-priority-indicator priority-indicator-urgent",
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

    let is_overdue = task.task_deadline.map(|ts| {
        let now = js_sys::Date::now() as i64;
        ts < now / 1000
    }).unwrap_or(false);

    view! {
        <div
            class=format!("kanban-card {} {}", if compact { "compact" } else { "" }, if is_overdue && has_deadline { "overdue" } else { "" })
            on:click=handle_click
        >
            <div class=priority_indicator_class></div>
            <div class="kanban-card-header">
                <div class="kanban-card-header-left">
                    <span class=format!("priority-badge {}", priority_class)>
                        {priority_label}
                    </span>
                </div>
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
                {if has_deadline {
                    view! {
                        <span class=format!("deadline-badge {}", if is_overdue { "overdue" } else { "" })>
                            <svg class="deadline-icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                                <circle cx="8" cy="8" r="6.5"/>
                                <path d="M8 4.5V8l2.5 1.5"/>
                            </svg>
                            {deadline_text.unwrap_or_default()}
                        </span>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {if has_team {
                    view! {
                        <span class="team-badge">
                            <svg class="team-icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                                <path d="M5.5 7a2 2 0 100-4 2 2 0 000 4zM10.5 7a2 2 0 100-4 2 2 0 000 4zM2 13c0-2.5 2-4 3.5-4s1.5.5 2.5.5 1-.5 2.5-.5 3.5 1.5 3.5 4"/>
                            </svg>
                            {team_name.unwrap_or_default()}
                        </span>
                    }.into_any()
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
    pub icon: &'static str,
}

pub fn get_kanban_columns() -> Vec<KanbanColumnConfig> {
    vec![
        KanbanColumnConfig {
            status: TaskStatus::Active,
            title: "To Do",
            color: "#3b82f6",
            icon: "M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4",
        },
        KanbanColumnConfig {
            status: TaskStatus::Paused,
            title: "In Progress",
            color: "#f59e0b",
            icon: "M13 10V3L4 14h7v7l9-11h-7z",
        },
        KanbanColumnConfig {
            status: TaskStatus::Completed,
            title: "Done",
            color: "#10b981",
            icon: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z",
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
    let task_count = column_tasks.len();
    let is_empty = task_count == 0;

    view! {
        <div class="kanban-column" style=format!("--column-color: {}", config.color)>
            <div class="kanban-column-header">
                <div class="kanban-column-header-left">
                    <span class="kanban-column-icon" style=format!("color: {}", config.color)>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <path d=config.icon/>
                        </svg>
                    </span>
                    <h2 class="kanban-column-title">{config.title}</h2>
                </div>
                <span class="kanban-column-count" style=format!("--column-color: {}", config.color)>
                    {task_count}
                </span>
            </div>
            <div class="kanban-column-content">
                {if is_empty {
                    view! {
                        <div class="kanban-column-empty">
                            <svg class="kanban-empty-icon" viewBox="0 0 48 48" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
                                <rect x="8" y="8" width="32" height="32" rx="4"/>
                                <path d="M16 20h16M16 28h8"/>
                            </svg>
                            <p class="kanban-empty-text">"No tasks"</p>
                        </div>
                    }.into_any()
                } else {
                    ().into_any()
                }}
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
